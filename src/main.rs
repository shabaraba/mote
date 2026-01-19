mod cli;
mod config;
mod error;
mod ignore;
mod storage;

use std::collections::{HashMap, HashSet};
use std::path::Path;

use clap::Parser;
use colored::*;

use cli::{Cli, Commands};
use config::Config;
use error::{MoteError, Result};
use ignore::{create_default_moteignore, IgnoreFilter};
use storage::{FileEntry, ObjectStore, Snapshot, SnapshotStore, StorageLocation};

fn main() {
    if let Err(e) = run() {
        eprintln!("{}: {}", "error".red().bold(), e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let project_root = std::env::current_dir()?;
    let config = Config::load()?;

    match cli.command {
        Commands::Init => cmd_init(&project_root, &config),
        Commands::Snapshot { message, trigger, auto } => {
            cmd_snapshot(&project_root, &config, message, trigger, auto)
        }
        Commands::SetupShell { shell } => cmd_setup_shell(&shell),
        Commands::Log { limit, oneline } => cmd_log(&project_root, limit, oneline),
        Commands::Show { snapshot_id } => cmd_show(&project_root, &snapshot_id),
        Commands::Diff {
            snapshot_id,
            snapshot_id2,
            content,
        } => cmd_diff(&project_root, &config, &snapshot_id, snapshot_id2, content),
        Commands::Restore {
            snapshot_id,
            file,
            force,
            dry_run,
        } => cmd_restore(&project_root, &config, &snapshot_id, file, force, dry_run),
    }
}

fn cmd_init(project_root: &Path, config: &Config) -> Result<()> {
    Config::save_default()?;
    let location = StorageLocation::init(project_root, config)?;
    create_default_moteignore(project_root)?;

    println!(
        "{} Initialized mote in {}",
        "✓".green().bold(),
        location.root().display()
    );
    println!("  Created {} for ignore patterns", ".moteignore".cyan());
    Ok(())
}

fn collect_files(
    project_root: &Path,
    config: &Config,
    object_store: &ObjectStore,
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

        match object_store.store_file(path) {
            Ok((hash, size)) => {
                files.push(FileEntry {
                    path: relative_path,
                    hash,
                    size,
                    mode: None,
                });
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

fn have_same_file_hashes(files1: &[FileEntry], files2: &[FileEntry]) -> bool {
    if files1.len() != files2.len() {
        return false;
    }
    let map: HashMap<_, _> = files1.iter().map(|f| (&f.path, &f.hash)).collect();
    files2.iter().all(|f| map.get(&f.path) == Some(&&f.hash))
}

fn cmd_snapshot(
    project_root: &Path,
    config: &Config,
    message: Option<String>,
    trigger: Option<String>,
    auto: bool,
) -> Result<()> {
    let location = match StorageLocation::find_existing(project_root) {
        Ok(loc) => loc,
        Err(_) if auto => return Ok(()),
        Err(e) => return Err(e),
    };
    let object_store = ObjectStore::new(location.objects_dir(), config.storage.compression_level);
    let snapshot_store = SnapshotStore::new(location.snapshots_dir());

    let files = collect_files(project_root, config, &object_store, auto);

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

    if config.snapshot.auto_cleanup {
        let removed = snapshot_store.cleanup(
            config.snapshot.max_snapshots,
            config.snapshot.max_age_days,
        )?;
        if removed > 0 && !auto {
            println!("  Cleaned up {} old snapshot(s)", removed);
        }
    }

    Ok(())
}

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

fn cmd_log(project_root: &Path, limit: usize, oneline: bool) -> Result<()> {
    let location = StorageLocation::find_existing(project_root)?;
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

fn cmd_show(project_root: &Path, snapshot_id: &str) -> Result<()> {
    let location = StorageLocation::find_existing(project_root)?;
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

fn cmd_diff(
    project_root: &Path,
    config: &Config,
    snapshot_id: &str,
    snapshot_id2: Option<String>,
    show_diff: bool,
) -> Result<()> {
    let location = StorageLocation::find_existing(project_root)?;
    let snapshot_store = SnapshotStore::new(location.snapshots_dir());
    let object_store = ObjectStore::new(location.objects_dir(), config.storage.compression_level);
    let snapshot1 = snapshot_store.find_by_id(snapshot_id)?;

    if let Some(ref id2) = snapshot_id2 {
        let snapshot2 = snapshot_store.find_by_id(id2)?;
        diff_snapshots(&snapshot1, &snapshot2, &object_store, show_diff)
    } else {
        diff_with_working_dir(project_root, config, &snapshot1, &object_store, show_diff)
    }
}

fn files_to_map(files: &[FileEntry]) -> HashMap<&str, &FileEntry> {
    files.iter().map(|f| (f.path.as_str(), f)).collect()
}

fn diff_snapshots(
    snapshot1: &Snapshot,
    snapshot2: &Snapshot,
    object_store: &ObjectStore,
    show_diff: bool,
) -> Result<()> {
    println!(
        "Comparing {} -> {}",
        snapshot1.short_id().cyan(),
        snapshot2.short_id().cyan()
    );
    println!();

    let files1 = files_to_map(&snapshot1.files);
    let files2 = files_to_map(&snapshot2.files);

    for (path, file2) in &files2 {
        if let Some(file1) = files1.get(path) {
            if file1.hash != file2.hash {
                println!("{}: {}", "Modified".yellow(), path);
                if show_diff {
                    show_content_diff(object_store, &file1.hash, &file2.hash)?;
                }
            }
        } else {
            println!("{}:   {}", "Added".green(), path);
        }
    }

    for path in files1.keys() {
        if !files2.contains_key(path) {
            println!("{}: {}", "Deleted".red(), path);
        }
    }
    Ok(())
}

fn diff_with_working_dir(
    project_root: &Path,
    config: &Config,
    snapshot: &Snapshot,
    object_store: &ObjectStore,
    show_diff: bool,
) -> Result<()> {
    println!(
        "Comparing {} -> working directory",
        snapshot.short_id().cyan()
    );
    println!();

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
            let current_hash = ObjectStore::compute_hash(&std::fs::read(path)?);
            if current_hash != snapshot_file.hash {
                println!("{}: {}", "Modified".yellow(), relative_path);
                if show_diff {
                    show_content_diff(object_store, &snapshot_file.hash, &current_hash)?;
                }
            }
        } else {
            println!("{}:   {}", "Added".green(), relative_path);
        }
    }

    for path in snapshot_files.keys() {
        if !current_files.contains(*path) {
            println!("{}: {}", "Deleted".red(), path);
        }
    }
    Ok(())
}

fn show_content_diff(object_store: &ObjectStore, hash1: &str, hash2: &str) -> Result<()> {
    let content1 = object_store.retrieve(hash1)?;
    let content2 = match object_store.retrieve(hash2) {
        Ok(c) => c,
        Err(MoteError::ObjectNotFound(_)) => return Ok(()),
        Err(e) => return Err(e),
    };

    let (Ok(text1), Ok(text2)) = (
        String::from_utf8(content1),
        String::from_utf8(content2),
    ) else {
        return Ok(());
    };

    let lines1: Vec<_> = text1.lines().collect();
    let lines2: Vec<_> = text2.lines().collect();

    println!("  ---");
    for (i, (l1, l2)) in lines1.iter().zip(lines2.iter()).enumerate() {
        if l1 != l2 {
            let ln = format!("{}:", i + 1).dimmed();
            println!("  {} {} {}", ln, "-".red(), l1.red());
            println!("  {} {} {}", ln, "+".green(), l2.green());
        }
    }

    for (i, line) in lines2.iter().enumerate().skip(lines1.len()) {
        println!(
            "  {} {} {}",
            format!("{}:", i + 1).dimmed(),
            "+".green(),
            line.green()
        );
    }

    for (i, line) in lines1.iter().enumerate().skip(lines2.len()) {
        println!(
            "  {} {} {}",
            format!("{}:", i + 1).dimmed(),
            "-".red(),
            line.red()
        );
    }
    println!("  ---");
    Ok(())
}

fn cmd_restore(
    project_root: &Path,
    config: &Config,
    snapshot_id: &str,
    file: Option<String>,
    force: bool,
    dry_run: bool,
) -> Result<()> {
    let location = StorageLocation::find_existing(project_root)?;
    let snapshot_store = SnapshotStore::new(location.snapshots_dir());
    let object_store = ObjectStore::new(location.objects_dir(), config.storage.compression_level);
    let snapshot = snapshot_store.find_by_id(snapshot_id)?;

    if let Some(ref file_path) = file {
        restore_single_file(project_root, &snapshot, &object_store, file_path, dry_run)
    } else {
        restore_all_files(
            project_root,
            config,
            &snapshot,
            &object_store,
            &snapshot_store,
            force,
            dry_run,
        )
    }
}

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

fn create_backup_snapshot(
    project_root: &Path,
    config: &Config,
    object_store: &ObjectStore,
    snapshot_store: &SnapshotStore,
    target_snapshot: &Snapshot,
) -> Result<()> {
    let files = collect_files(project_root, config, object_store, true);
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

fn restore_all_files(
    project_root: &Path,
    config: &Config,
    snapshot: &Snapshot,
    object_store: &ObjectStore,
    snapshot_store: &SnapshotStore,
    force: bool,
    dry_run: bool,
) -> Result<()> {
    if !force && !dry_run {
        create_backup_snapshot(project_root, config, object_store, snapshot_store, snapshot)?;
    }

    let (restored, skipped) =
        restore_files(project_root, snapshot, object_store, force, dry_run)?;

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

fn restore_files(
    project_root: &Path,
    snapshot: &Snapshot,
    object_store: &ObjectStore,
    force: bool,
    dry_run: bool,
) -> Result<(u32, u32)> {
    let mut restored = 0;
    let mut skipped = 0;

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

        if dest.exists() && !force {
            let current_hash = ObjectStore::compute_hash(&std::fs::read(&dest)?);
            if current_hash != file.hash {
                println!(
                    "{}: {} (use --force to overwrite)",
                    "Skipped".yellow(),
                    file.path
                );
                skipped += 1;
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
