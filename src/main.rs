mod cli;
mod config;
mod error;
mod ignore;
mod storage;

use clap::Parser;
use colored::*;
use std::env;
use std::path::PathBuf;

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
    let project_root = env::current_dir()?;
    let config = Config::load()?;

    match cli.command {
        Commands::Init => cmd_init(&project_root, &config),
        Commands::Snapshot { message, trigger } => {
            cmd_snapshot(&project_root, &config, message, trigger)
        }
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

fn cmd_init(project_root: &PathBuf, config: &Config) -> Result<()> {
    Config::save_default()?;

    let location = StorageLocation::init(project_root, config)?;
    create_default_moteignore(project_root)?;

    println!(
        "{} Initialized mote in {}",
        "✓".green().bold(),
        location.root().display()
    );
    println!(
        "  Created {} for ignore patterns",
        ".moteignore".cyan()
    );

    Ok(())
}

fn cmd_snapshot(
    project_root: &PathBuf,
    config: &Config,
    message: Option<String>,
    trigger: Option<String>,
) -> Result<()> {
    let location = StorageLocation::find_existing(project_root)?;
    let object_store = ObjectStore::new(location.objects_dir(), config.storage.compression_level);
    let snapshot_store = SnapshotStore::new(location.snapshots_dir());
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
            Err(e) => {
                eprintln!(
                    "{}: Failed to store {}: {}",
                    "warning".yellow(),
                    relative_path,
                    e
                );
            }
        }
    }

    if files.is_empty() {
        println!("{} No files to snapshot", "!".yellow().bold());
        return Ok(());
    }

    let snapshot = Snapshot::new(files, message.clone(), trigger);
    snapshot_store.save(&snapshot)?;

    println!(
        "{} Created snapshot {} ({} files)",
        "✓".green().bold(),
        snapshot.short_id().cyan(),
        snapshot.file_count()
    );

    if let Some(msg) = message {
        println!("  Message: {}", msg);
    }

    if config.snapshot.auto_cleanup {
        let removed = snapshot_store.cleanup(
            config.snapshot.max_snapshots,
            config.snapshot.max_age_days,
        )?;
        if removed > 0 {
            println!("  Cleaned up {} old snapshot(s)", removed);
        }
    }

    Ok(())
}

fn cmd_log(project_root: &PathBuf, limit: usize, oneline: bool) -> Result<()> {
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
            println!("Date:    {}", snapshot.timestamp.format("%Y-%m-%d %H:%M:%S %Z"));
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

fn cmd_show(project_root: &PathBuf, snapshot_id: &str) -> Result<()> {
    let location = StorageLocation::find_existing(project_root)?;
    let snapshot_store = SnapshotStore::new(location.snapshots_dir());

    let snapshot = snapshot_store.find_by_id(snapshot_id)?;

    println!("{} {}", "snapshot".yellow(), snapshot.id.cyan());
    println!("Date:    {}", snapshot.timestamp.format("%Y-%m-%d %H:%M:%S %Z"));
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
        println!(
            "  {} ({} bytes)",
            file.path.cyan(),
            file.size
        );
    }

    Ok(())
}

fn cmd_diff(
    project_root: &PathBuf,
    config: &Config,
    snapshot_id: &str,
    snapshot_id2: Option<String>,
    show_content: bool,
) -> Result<()> {
    let location = StorageLocation::find_existing(project_root)?;
    let snapshot_store = SnapshotStore::new(location.snapshots_dir());
    let object_store = ObjectStore::new(location.objects_dir(), config.storage.compression_level);

    let snapshot1 = snapshot_store.find_by_id(snapshot_id)?;

    if let Some(ref id2) = snapshot_id2 {
        let snapshot2 = snapshot_store.find_by_id(id2)?;
        diff_snapshots(&snapshot1, &snapshot2, &object_store, show_content)?;
    } else {
        diff_with_working_dir(project_root, config, &snapshot1, &object_store, show_content)?;
    }

    Ok(())
}

fn diff_snapshots(
    snapshot1: &Snapshot,
    snapshot2: &Snapshot,
    object_store: &ObjectStore,
    show_content: bool,
) -> Result<()> {
    println!(
        "Comparing {} -> {}",
        snapshot1.short_id().cyan(),
        snapshot2.short_id().cyan()
    );
    println!();

    let files1: std::collections::HashMap<_, _> = snapshot1
        .files
        .iter()
        .map(|f| (f.path.clone(), f))
        .collect();
    let files2: std::collections::HashMap<_, _> = snapshot2
        .files
        .iter()
        .map(|f| (f.path.clone(), f))
        .collect();

    for (path, file2) in &files2 {
        if let Some(file1) = files1.get(path) {
            if file1.hash != file2.hash {
                println!("{}: {}", "Modified".yellow(), path);
                if show_content {
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
    project_root: &PathBuf,
    config: &Config,
    snapshot: &Snapshot,
    object_store: &ObjectStore,
    show_content: bool,
) -> Result<()> {
    println!(
        "Comparing {} -> working directory",
        snapshot.short_id().cyan()
    );
    println!();

    let ignore_filter = IgnoreFilter::new(project_root, &config.ignore.ignore_file);

    let snapshot_files: std::collections::HashMap<_, _> = snapshot
        .files
        .iter()
        .map(|f| (f.path.clone(), f))
        .collect();

    let mut current_files = std::collections::HashSet::new();

    for entry in ignore_filter.walk_files(project_root) {
        let path = entry.path();
        let relative_path = path
            .strip_prefix(project_root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        current_files.insert(relative_path.clone());

        if let Some(snapshot_file) = snapshot_files.get(&relative_path) {
            let current_hash = ObjectStore::compute_hash(&std::fs::read(path)?);
            if current_hash != snapshot_file.hash {
                println!("{}: {}", "Modified".yellow(), relative_path);
                if show_content {
                    show_content_diff(object_store, &snapshot_file.hash, &current_hash)?;
                }
            }
        } else {
            println!("{}:   {}", "Added".green(), relative_path);
        }
    }

    for path in snapshot_files.keys() {
        if !current_files.contains(path) {
            println!("{}: {}", "Deleted".red(), path);
        }
    }

    Ok(())
}

fn show_content_diff(object_store: &ObjectStore, hash1: &str, hash2: &str) -> Result<()> {
    let content1 = object_store.retrieve(hash1)?;
    let content2_result = object_store.retrieve(hash2);

    let content2 = match content2_result {
        Ok(c) => c,
        Err(MoteError::ObjectNotFound(_)) => return Ok(()),
        Err(e) => return Err(e),
    };

    if let (Ok(text1), Ok(text2)) = (
        String::from_utf8(content1.clone()),
        String::from_utf8(content2.clone()),
    ) {
        let lines1: Vec<_> = text1.lines().collect();
        let lines2: Vec<_> = text2.lines().collect();

        println!("  ---");
        for (i, (l1, l2)) in lines1.iter().zip(lines2.iter()).enumerate() {
            if l1 != l2 {
                println!("  {} {} {}", format!("{}:", i + 1).dimmed(), "-".red(), l1.red());
                println!("  {} {} {}", format!("{}:", i + 1).dimmed(), "+".green(), l2.green());
            }
        }

        for (i, line) in lines2.iter().enumerate().skip(lines1.len()) {
            println!("  {} {} {}", format!("{}:", i + 1).dimmed(), "+".green(), line.green());
        }

        for (i, line) in lines1.iter().enumerate().skip(lines2.len()) {
            println!("  {} {} {}", format!("{}:", i + 1).dimmed(), "-".red(), line.red());
        }
        println!("  ---");
    }

    Ok(())
}

fn cmd_restore(
    project_root: &PathBuf,
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
        restore_single_file(
            project_root,
            &snapshot,
            &object_store,
            file_path,
            dry_run,
        )
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
    project_root: &PathBuf,
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

fn restore_all_files(
    project_root: &PathBuf,
    config: &Config,
    snapshot: &Snapshot,
    object_store: &ObjectStore,
    snapshot_store: &SnapshotStore,
    force: bool,
    dry_run: bool,
) -> Result<()> {
    if !force && !dry_run {
        let ignore_filter = IgnoreFilter::new(project_root, &config.ignore.ignore_file);
        let mut files = Vec::new();

        for entry in ignore_filter.walk_files(project_root) {
            let path = entry.path();
            let relative_path = path
                .strip_prefix(project_root)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();

            if let Ok((hash, size)) = object_store.store_file(path) {
                files.push(FileEntry {
                    path: relative_path,
                    hash,
                    size,
                    mode: None,
                });
            }
        }

        if !files.is_empty() {
            let backup = Snapshot::new(
                files,
                Some(format!("Backup before restore to {}", snapshot.short_id())),
                Some("auto-backup".to_string()),
            );
            snapshot_store.save(&backup)?;
            println!(
                "{} Created backup snapshot: {}",
                "✓".green().bold(),
                backup.short_id().cyan()
            );
        }
    }

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
            Ok(_) => {
                restored += 1;
            }
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

    if dry_run {
        println!(
            "\n{} Would restore {} file(s)",
            "dry-run".cyan().bold(),
            restored
        );
    } else {
        println!(
            "\n{} Restored {} file(s)",
            "✓".green().bold(),
            restored
        );
        if skipped > 0 {
            println!("  Skipped {} modified file(s)", skipped);
        }
    }

    Ok(())
}
