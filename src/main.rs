mod cli;
mod config;
mod error;
mod ignore;
mod storage;

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use clap::Parser;
use colored::*;
use similar::{ChangeTag, TextDiff};

use cli::{Cli, Commands};
use config::Config;
use error::{MoteError, Result};
use ignore::{create_default_moteignore, IgnoreFilter};
use storage::{
    FileEntry, Index, IndexEntry, ObjectStore, Snapshot, SnapshotStore, StorageLocation,
};

/// Context holding common parameters passed to command functions.
struct Context<'a> {
    /// The project root directory.
    project_root: &'a Path,
    /// The loaded configuration.
    config: &'a Config,
    /// Optional custom storage directory.
    storage_dir: Option<&'a Path>,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{}: {}", "error".red().bold(), e);
        std::process::exit(1);
    }
}

/// Main entry point for command execution.
/// Parses CLI arguments, resolves paths, and dispatches to appropriate command handlers.
fn run() -> Result<()> {
    let cli = Cli::parse();
    let project_root = cli
        .project_root
        .clone()
        .unwrap_or_else(|| std::env::current_dir().expect("Failed to get current directory"));
    let mut config = Config::load()?;

    let resolved_ignore_file = cli.ignore_file.as_ref().map(|path| {
        if path.is_absolute() {
            path.clone()
        } else {
            project_root.join(path)
        }
    });

    if let Some(ignore_file) = &resolved_ignore_file {
        config.ignore.ignore_file = ignore_file
            .to_str()
            .ok_or_else(|| MoteError::ConfigRead("Invalid ignore file path".to_string()))?
            .to_string();
    }

    let resolved_storage_dir = cli.storage_dir.as_ref().map(|path| {
        if path.is_absolute() {
            path.clone()
        } else {
            project_root.join(path)
        }
    });

    let ctx = Context {
        project_root: &project_root,
        config: &config,
        storage_dir: resolved_storage_dir.as_deref(),
    };

    match cli.command {
        Commands::Init => cmd_init(&ctx),
        Commands::Snapshot {
            message,
            trigger,
            auto,
        } => cmd_snapshot(&ctx, message, trigger, auto),
        Commands::SetupShell { shell } => cmd_setup_shell(&shell),
        Commands::Log { limit, oneline } => cmd_log(&ctx, limit, oneline),
        Commands::Show { snapshot_id } => cmd_show(&ctx, &snapshot_id),
        Commands::Diff {
            snapshot_id,
            snapshot_id2,
            name_only,
            output,
            unified,
        } => cmd_diff(&ctx, snapshot_id, snapshot_id2, name_only, output, unified),
        Commands::Restore {
            snapshot_id,
            file,
            force,
            dry_run,
        } => cmd_restore(&ctx, &snapshot_id, file, force, dry_run),
    }
}

/// Initialize mote in the project directory.
/// Creates storage directories and default ignore file.
fn cmd_init(ctx: &Context) -> Result<()> {
    Config::save_default()?;
    let location = StorageLocation::init(ctx.project_root, ctx.config, ctx.storage_dir)?;
    create_default_moteignore(ctx.project_root)?;

    println!(
        "{} Initialized mote in {}",
        "✓".green().bold(),
        location.root().display()
    );
    println!("  Created {} for ignore patterns", ".moteignore".cyan());
    Ok(())
}

/// Collect all files from the project directory, respecting ignore rules.
/// Uses the index cache to skip unchanged files for performance.
fn collect_files(
    project_root: &Path,
    config: &Config,
    object_store: &ObjectStore,
    index: &mut Index,
    quiet: bool,
) -> Vec<FileEntry> {
    let ignore_filter = IgnoreFilter::new(project_root, &config.ignore.ignore_file);
    let mut files = Vec::new();

    for entry in ignore_filter.walk_files(project_root) {
        let path = entry.path();
        let relative_path = path
            .strip_prefix(project_root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        let metadata = match fs::metadata(path) {
            Ok(m) => m,
            Err(e) if !quiet => {
                eprintln!(
                    "{}: Failed to read metadata for {}: {}",
                    "warning".yellow(),
                    relative_path,
                    e
                );
                continue;
            }
            Err(_) => continue,
        };

        let mtime = match metadata.modified() {
            Ok(t) => t,
            Err(e) if !quiet => {
                eprintln!(
                    "{}: Failed to get mtime for {}: {}",
                    "warning".yellow(),
                    relative_path,
                    e
                );
                continue;
            }
            Err(_) => continue,
        };

        let size = metadata.len();

        if let Some(cached_entry) = index.is_unchanged(&relative_path, mtime, size) {
            files.push(FileEntry {
                path: relative_path,
                hash: cached_entry.hash.clone(),
                size: cached_entry.size,
                mode: None,
            });
            continue;
        }

        match object_store.store_file(path) {
            Ok((hash, file_size)) => {
                let entry = FileEntry {
                    path: relative_path.clone(),
                    hash: hash.clone(),
                    size: file_size,
                    mode: None,
                };

                index.insert(IndexEntry {
                    path: relative_path,
                    hash,
                    size: file_size,
                    mtime,
                });

                files.push(entry);
            }
            Err(e) if !quiet => {
                eprintln!(
                    "{}: Failed to store {}: {}",
                    "warning".yellow(),
                    relative_path,
                    e
                );
            }
            _ => {}
        }
    }
    files
}

/// Check if two file lists have identical content hashes.
/// Used to skip creating duplicate snapshots in auto mode.
fn have_same_file_hashes(files1: &[FileEntry], files2: &[FileEntry]) -> bool {
    if files1.len() != files2.len() {
        return false;
    }
    let map: HashMap<_, _> = files1.iter().map(|f| (&f.path, &f.hash)).collect();
    files2.iter().all(|f| map.get(&f.path) == Some(&&f.hash))
}

/// Create a new snapshot of the project files.
/// In auto mode, skips if no changes detected or no storage initialized.
/// Auto-initializes storage if custom storage_dir is specified.
fn cmd_snapshot(
    ctx: &Context,
    message: Option<String>,
    trigger: Option<String>,
    auto: bool,
) -> Result<()> {
    let location = match StorageLocation::find_existing(ctx.project_root, ctx.storage_dir) {
        Ok(loc) => loc,
        Err(MoteError::NotInitialized) if ctx.storage_dir.is_some() => {
            // Auto-initialize when custom storage_dir is specified
            StorageLocation::init(ctx.project_root, ctx.config, ctx.storage_dir)?
        }
        Err(_) if auto => return Ok(()),
        Err(e) => return Err(e),
    };
    let object_store =
        ObjectStore::new(location.objects_dir(), ctx.config.storage.compression_level);
    let snapshot_store = SnapshotStore::new(location.snapshots_dir());

    let mut index = Index::load(&location.index_path())?;
    let files = collect_files(
        ctx.project_root,
        ctx.config,
        &object_store,
        &mut index,
        auto,
    );
    index.save(&location.index_path())?;

    if files.is_empty() {
        if !auto {
            println!("{} No files to snapshot", "!".yellow().bold());
        }
        return Ok(());
    }

    if auto {
        if let Ok(snapshots) = snapshot_store.list() {
            if let Some(latest) = snapshots.first() {
                if have_same_file_hashes(&latest.files, &files) {
                    return Ok(());
                }
            }
        }
    }

    let snapshot = Snapshot::new(files, message.clone(), trigger);
    snapshot_store.save(&snapshot)?;

    if !auto {
        println!(
            "{} Created snapshot {} ({} files)",
            "✓".green().bold(),
            snapshot.short_id().cyan(),
            snapshot.file_count()
        );
        if let Some(msg) = message {
            println!("  Message: {}", msg);
        }
    }

    if ctx.config.snapshot.auto_cleanup {
        let removed = snapshot_store.cleanup(
            ctx.config.snapshot.max_snapshots,
            ctx.config.snapshot.max_age_days,
        )?;
        if removed > 0 && !auto {
            println!("  Cleaned up {} old snapshot(s)", removed);
        }
    }

    Ok(())
}

/// Print shell integration script for auto-snapshot hooks.
/// Supports bash, zsh, and fish shells.
fn cmd_setup_shell(shell: &str) -> Result<()> {
    let script = match shell {
        "bash" | "zsh" => include_str!("../scripts/shell_integration.sh"),
        "fish" => include_str!("../scripts/shell_integration.fish"),
        _ => {
            return Err(MoteError::ConfigRead(format!(
                "Unsupported shell: {}. Use bash, zsh, or fish.",
                shell
            )));
        }
    };
    println!("{}", script);
    Ok(())
}

/// Display snapshot history with optional formatting.
/// Shows up to `limit` most recent snapshots.
/// Auto-initializes storage if custom storage_dir is specified.
fn cmd_log(ctx: &Context, limit: usize, oneline: bool) -> Result<()> {
    let location = match StorageLocation::find_existing(ctx.project_root, ctx.storage_dir) {
        Ok(loc) => loc,
        Err(MoteError::NotInitialized) if ctx.storage_dir.is_some() => {
            StorageLocation::init(ctx.project_root, ctx.config, ctx.storage_dir)?
        }
        Err(e) => return Err(e),
    };
    let snapshot_store = SnapshotStore::new(location.snapshots_dir());
    let snapshots = snapshot_store.list()?;

    if snapshots.is_empty() {
        println!("{} No snapshots yet", "!".yellow().bold());
        return Ok(());
    }

    for snapshot in snapshots.into_iter().take(limit) {
        if oneline {
            println!(
                "{} {}  {}  ({} files)",
                snapshot.short_id().cyan(),
                snapshot.timestamp.format("%Y-%m-%d %H:%M:%S"),
                snapshot.message.as_deref().unwrap_or("-").dimmed(),
                snapshot.file_count()
            );
        } else {
            println!("{} {}", "snapshot".yellow(), snapshot.short_id().cyan());
            println!(
                "Date:    {}",
                snapshot.timestamp.format("%Y-%m-%d %H:%M:%S %Z")
            );
            if let Some(ref msg) = snapshot.message {
                println!("Message: {}", msg);
            }
            if let Some(ref trigger) = snapshot.trigger {
                println!("Trigger: {}", trigger);
            }
            println!("Files:   {}", snapshot.file_count());
            println!();
        }
    }
    Ok(())
}

/// Show detailed information about a specific snapshot.
/// Includes metadata and file list.
/// Auto-initializes storage if custom storage_dir is specified.
fn cmd_show(ctx: &Context, snapshot_id: &str) -> Result<()> {
    let location = match StorageLocation::find_existing(ctx.project_root, ctx.storage_dir) {
        Ok(loc) => loc,
        Err(MoteError::NotInitialized) if ctx.storage_dir.is_some() => {
            StorageLocation::init(ctx.project_root, ctx.config, ctx.storage_dir)?
        }
        Err(e) => return Err(e),
    };
    let snapshot_store = SnapshotStore::new(location.snapshots_dir());
    let snapshot = snapshot_store.find_by_id(snapshot_id)?;

    println!("{} {}", "snapshot".yellow(), snapshot.id.cyan());
    println!(
        "Date:    {}",
        snapshot.timestamp.format("%Y-%m-%d %H:%M:%S %Z")
    );
    if let Some(ref msg) = snapshot.message {
        println!("Message: {}", msg);
    }
    if let Some(ref trigger) = snapshot.trigger {
        println!("Trigger: {}", trigger);
    }
    println!("Files:   {}", snapshot.file_count());
    println!();
    println!("{}:", "Files".bold());

    for file in &snapshot.files {
        println!("  {} ({} bytes)", file.path.cyan(), file.size);
    }
    Ok(())
}

/// Show differences between snapshots or working directory.
/// Compares two snapshots, or a snapshot with current working directory.
/// Auto-initializes storage if custom storage_dir is specified.
fn cmd_diff(
    ctx: &Context,
    snapshot_id: Option<String>,
    snapshot_id2: Option<String>,
    name_only: bool,
    output: Option<String>,
    unified: usize,
) -> Result<()> {
    let location = match StorageLocation::find_existing(ctx.project_root, ctx.storage_dir) {
        Ok(loc) => loc,
        Err(MoteError::NotInitialized) if ctx.storage_dir.is_some() => {
            StorageLocation::init(ctx.project_root, ctx.config, ctx.storage_dir)?
        }
        Err(e) => return Err(e),
    };
    let snapshot_store = SnapshotStore::new(location.snapshots_dir());
    let object_store =
        ObjectStore::new(location.objects_dir(), ctx.config.storage.compression_level);

    let snapshot_id = match snapshot_id {
        Some(id) => id,
        None => {
            let snapshots = snapshot_store.list()?;
            if snapshots.is_empty() {
                return Err(MoteError::ConfigRead("No snapshots found".to_string()));
            }
            snapshots.first().unwrap().id.clone()
        }
    };

    let snapshot1 = snapshot_store.find_by_id(&snapshot_id)?;

    let mut diff_output = String::new();

    if let Some(ref id2) = snapshot_id2 {
        let snapshot2 = snapshot_store.find_by_id(id2)?;
        diff_snapshots(
            &snapshot1,
            &snapshot2,
            &object_store,
            name_only,
            unified,
            &mut diff_output,
        )?;
    } else {
        diff_with_working_dir(
            ctx.project_root,
            ctx.config,
            &snapshot1,
            &object_store,
            name_only,
            unified,
            &mut diff_output,
        )?;
    }

    if let Some(output_file) = output {
        fs::write(&output_file, &diff_output)?;
        println!("Diff written to {}", output_file.cyan());
    } else {
        print!("{}", diff_output);
    }

    Ok(())
}

/// Convert file list to a hashmap for efficient lookup by path.
fn files_to_map(files: &[FileEntry]) -> HashMap<&str, &FileEntry> {
    files.iter().map(|f| (f.path.as_str(), f)).collect()
}

/// Generate diff between two snapshots.
/// Outputs unified diff format or file names only.
fn diff_snapshots(
    snapshot1: &Snapshot,
    snapshot2: &Snapshot,
    object_store: &ObjectStore,
    name_only: bool,
    unified: usize,
    output: &mut String,
) -> Result<()> {
    use std::fmt::Write;

    writeln!(
        output,
        "Comparing {} -> {}",
        snapshot1.short_id(),
        snapshot2.short_id()
    )
    .unwrap();
    writeln!(output).unwrap();

    let files1 = files_to_map(&snapshot1.files);
    let files2 = files_to_map(&snapshot2.files);

    for (path, file2) in &files2 {
        if let Some(file1) = files1.get(path) {
            if file1.hash != file2.hash {
                if name_only {
                    writeln!(output, "M\t{}", path).unwrap();
                } else {
                    generate_unified_diff(
                        object_store,
                        path,
                        &file1.hash,
                        &file2.hash,
                        unified,
                        output,
                    )?;
                }
            }
        } else if name_only {
            writeln!(output, "A\t{}", path).unwrap();
        } else {
            generate_unified_diff(object_store, path, "", &file2.hash, unified, output)?;
        }
    }

    for path in files1.keys() {
        if !files2.contains_key(path) {
            if name_only {
                writeln!(output, "D\t{}", path).unwrap();
            } else {
                let file1 = files1.get(path).unwrap();
                generate_unified_diff(object_store, path, &file1.hash, "", unified, output)?;
            }
        }
    }
    Ok(())
}

/// Generate diff between a snapshot and current working directory.
/// Respects ignore rules when scanning working directory.
fn diff_with_working_dir(
    project_root: &Path,
    config: &Config,
    snapshot: &Snapshot,
    object_store: &ObjectStore,
    name_only: bool,
    unified: usize,
    output: &mut String,
) -> Result<()> {
    use std::fmt::Write;

    writeln!(
        output,
        "Comparing {} -> working directory",
        snapshot.short_id()
    )
    .unwrap();
    writeln!(output).unwrap();

    let ignore_filter = IgnoreFilter::new(project_root, &config.ignore.ignore_file);
    let snapshot_files = files_to_map(&snapshot.files);
    let mut current_files = HashSet::new();

    for entry in ignore_filter.walk_files(project_root) {
        let path = entry.path();
        let relative_path = path
            .strip_prefix(project_root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        current_files.insert(relative_path.clone());

        if let Some(snapshot_file) = snapshot_files.get(relative_path.as_str()) {
            let current_content = fs::read(path)?;
            let current_hash = ObjectStore::compute_hash(&current_content);
            if current_hash != snapshot_file.hash {
                if name_only {
                    writeln!(output, "M\t{}", relative_path).unwrap();
                } else {
                    generate_unified_diff_with_content(
                        object_store,
                        &relative_path,
                        &snapshot_file.hash,
                        &current_content,
                        unified,
                        output,
                    )?;
                }
            }
        } else if name_only {
            writeln!(output, "A\t{}", relative_path).unwrap();
        } else {
            let current_content = fs::read(path)?;
            generate_unified_diff_with_content(
                object_store,
                &relative_path,
                "",
                &current_content,
                unified,
                output,
            )?;
        }
    }

    for path in snapshot_files.keys() {
        if !current_files.contains(*path) {
            if name_only {
                writeln!(output, "D\t{}", path).unwrap();
            } else {
                let file = snapshot_files.get(path).unwrap();
                generate_unified_diff_with_content(
                    object_store,
                    path,
                    &file.hash,
                    &[],
                    unified,
                    output,
                )?;
            }
        }
    }
    Ok(())
}

/// Generate unified diff for a file between two content hashes.
/// Retrieves file contents from object store.
fn generate_unified_diff(
    object_store: &ObjectStore,
    path: &str,
    hash1: &str,
    hash2: &str,
    context_lines: usize,
    output: &mut String,
) -> Result<()> {
    let content2 = if hash2.is_empty() {
        Vec::new()
    } else {
        match object_store.retrieve(hash2) {
            Ok(c) => c,
            Err(MoteError::ObjectNotFound(hash)) => {
                eprintln!(
                    "{}: Object not found for {}: {}",
                    "warning".yellow(),
                    path,
                    hash
                );
                return Ok(());
            }
            Err(e) => return Err(e),
        }
    };

    generate_unified_diff_with_content(object_store, path, hash1, &content2, context_lines, output)
}

/// Generate unified diff for a file with explicit content.
/// Used when comparing with working directory files.
fn generate_unified_diff_with_content(
    object_store: &ObjectStore,
    path: &str,
    hash1: &str,
    content2: &[u8],
    context_lines: usize,
    output: &mut String,
) -> Result<()> {
    use std::fmt::Write;

    let content1 = if hash1.is_empty() {
        Vec::new()
    } else {
        object_store.retrieve(hash1)?
    };

    let text1 = String::from_utf8_lossy(&content1);
    let text2 = String::from_utf8_lossy(content2);

    if text1.is_empty() && text2.is_empty() {
        return Ok(());
    }

    let diff = TextDiff::from_lines(&text1, &text2);

    writeln!(output, "diff --mote a/{} b/{}", path, path).unwrap();
    writeln!(output, "--- a/{}", path).unwrap();
    writeln!(output, "+++ b/{}", path).unwrap();

    for hunk in diff
        .unified_diff()
        .context_radius(context_lines)
        .iter_hunks()
    {
        write!(output, "{}", hunk.header()).unwrap();
        for change in hunk.iter_changes() {
            let sign = match change.tag() {
                ChangeTag::Delete => "-",
                ChangeTag::Insert => "+",
                ChangeTag::Equal => " ",
            };
            write!(output, "{}{}", sign, change.value()).unwrap();
        }
    }

    writeln!(output).unwrap();
    Ok(())
}

/// Restore files from a snapshot.
/// Can restore entire snapshot or a specific file.
/// Auto-initializes storage if custom storage_dir is specified.
fn cmd_restore(
    ctx: &Context,
    snapshot_id: &str,
    file: Option<String>,
    force: bool,
    dry_run: bool,
) -> Result<()> {
    let location = match StorageLocation::find_existing(ctx.project_root, ctx.storage_dir) {
        Ok(loc) => loc,
        Err(MoteError::NotInitialized) if ctx.storage_dir.is_some() => {
            StorageLocation::init(ctx.project_root, ctx.config, ctx.storage_dir)?
        }
        Err(e) => return Err(e),
    };
    let snapshot_store = SnapshotStore::new(location.snapshots_dir());
    let object_store =
        ObjectStore::new(location.objects_dir(), ctx.config.storage.compression_level);
    let snapshot = snapshot_store.find_by_id(snapshot_id)?;

    if let Some(ref file_path) = file {
        restore_single_file(
            ctx.project_root,
            &snapshot,
            &object_store,
            file_path,
            dry_run,
        )
    } else {
        let mut index = Index::load(&location.index_path())?;
        let result = restore_all_files(
            ctx.project_root,
            ctx.config,
            &snapshot,
            &object_store,
            &snapshot_store,
            &mut index,
            force,
            dry_run,
        );
        if result.is_ok() {
            index.save(&location.index_path())?;
        }
        result
    }
}

/// Restore a single file from a snapshot.
/// Shows dry-run output if requested.
fn restore_single_file(
    project_root: &Path,
    snapshot: &Snapshot,
    object_store: &ObjectStore,
    file_path: &str,
    dry_run: bool,
) -> Result<()> {
    let file_entry = snapshot
        .find_file(file_path)
        .ok_or_else(|| MoteError::FileNotFoundInSnapshot(file_path.to_string()))?;

    let dest = project_root.join(&file_entry.path);

    if dry_run {
        println!(
            "{} Would restore: {} ({} bytes)",
            "dry-run".cyan().bold(),
            file_entry.path,
            file_entry.size
        );
    } else {
        object_store.restore_file(&file_entry.hash, &dest)?;
        println!(
            "{} Restored: {}",
            "✓".green().bold(),
            file_entry.path.cyan()
        );
    }
    Ok(())
}

/// Create automatic backup snapshot before restore operation.
/// Captures current state to allow undo.
fn create_backup_snapshot(
    project_root: &Path,
    config: &Config,
    object_store: &ObjectStore,
    snapshot_store: &SnapshotStore,
    target_snapshot: &Snapshot,
    index: &mut Index,
) -> Result<()> {
    let files = collect_files(project_root, config, object_store, index, true);
    if files.is_empty() {
        return Ok(());
    }

    let backup = Snapshot::new(
        files,
        Some(format!(
            "Backup before restore to {}",
            target_snapshot.short_id()
        )),
        Some("auto-backup".to_string()),
    );
    snapshot_store.save(&backup)?;
    println!(
        "{} Created backup snapshot: {}",
        "✓".green().bold(),
        backup.short_id().cyan()
    );
    Ok(())
}

/// Restore all files from a snapshot.
/// Creates backup unless force flag is set.
#[allow(clippy::too_many_arguments)]
fn restore_all_files(
    project_root: &Path,
    config: &Config,
    snapshot: &Snapshot,
    object_store: &ObjectStore,
    snapshot_store: &SnapshotStore,
    index: &mut Index,
    force: bool,
    dry_run: bool,
) -> Result<()> {
    if !force && !dry_run {
        create_backup_snapshot(
            project_root,
            config,
            object_store,
            snapshot_store,
            snapshot,
            index,
        )?;
    }

    let (restored, skipped) = restore_files(project_root, snapshot, object_store, force, dry_run)?;

    if dry_run {
        println!(
            "\n{} Would restore {} file(s)",
            "dry-run".cyan().bold(),
            restored
        );
    } else {
        println!("\n{} Restored {} file(s)", "✓".green().bold(), restored);
        if skipped > 0 {
            println!("  Skipped {} modified file(s)", skipped);
        }
    }
    Ok(())
}

/// Restore files from snapshot to disk.
/// Returns count of restored and skipped files.
fn restore_files(
    project_root: &Path,
    snapshot: &Snapshot,
    object_store: &ObjectStore,
    _force: bool,
    dry_run: bool,
) -> Result<(u32, u32)> {
    let mut restored = 0;
    let skipped = 0;

    for file in &snapshot.files {
        let dest = project_root.join(&file.path);

        if dry_run {
            println!(
                "{} Would restore: {} ({} bytes)",
                "dry-run".cyan().bold(),
                file.path,
                file.size
            );
            restored += 1;
            continue;
        }

        if dest.exists() {
            let current_hash = ObjectStore::compute_hash(&std::fs::read(&dest)?);
            if current_hash == file.hash {
                // Already in correct state, skip restore
                continue;
            }
        }

        match object_store.restore_file(&file.hash, &dest) {
            Ok(_) => restored += 1,
            Err(e) => {
                eprintln!(
                    "{}: Failed to restore {}: {}",
                    "warning".yellow(),
                    file.path,
                    e
                );
            }
        }
    }
    Ok((restored, skipped))
}
