use std::path::Path;

use colored::*;

use super::collect::collect_files;
use crate::commands::CommandContext;
use crate::error::{MoteError, Result};
use crate::storage::{Index, ObjectStore, Snapshot, SnapshotStore, StorageLocation};

pub fn cmd_restore(
    ctx: &CommandContext,
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
    let object_store = ObjectStore::new(location.objects_dir());
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
            &ctx.ignore_file_path,
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

fn restore_single_file(
    project_root: &Path,
    snapshot: &Snapshot,
    object_store: &ObjectStore,
    file_path: &str,
    dry_run: bool,
) -> Result<()> {
    // Convert absolute path to relative path if necessary
    let file_path_buf = Path::new(file_path);
    let relative_path = if file_path_buf.is_absolute() {
        file_path_buf
            .strip_prefix(project_root)
            .unwrap_or(file_path_buf)
            .to_string_lossy()
            .to_string()
    } else {
        file_path.to_string()
    };

    let dest = project_root.join(&relative_path);

    match snapshot.find_file(&relative_path) {
        Some(file_entry) => {
            // File exists in snapshot - restore it
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
        }
        None => {
            // File doesn't exist in snapshot - delete it if it exists
            if dest.exists() {
                if dry_run {
                    println!(
                        "{} Would delete: {} (not in snapshot)",
                        "dry-run".cyan().bold(),
                        file_path
                    );
                } else {
                    std::fs::remove_file(&dest)?;
                    println!(
                        "{} Deleted: {} (not in snapshot)",
                        "✓".green().bold(),
                        file_path.cyan()
                    );
                }
            } else {
                println!(
                    "{} File does not exist: {}",
                    "info".blue().bold(),
                    file_path
                );
            }
        }
    }
    Ok(())
}

fn create_backup_snapshot(
    project_root: &Path,
    ignore_file_path: &Path,
    object_store: &ObjectStore,
    snapshot_store: &SnapshotStore,
    target_snapshot: &Snapshot,
    index: &mut Index,
) -> Result<()> {
    let files = collect_files(project_root, ignore_file_path, object_store, index, true);
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

#[allow(clippy::too_many_arguments)]
fn restore_all_files(
    project_root: &Path,
    ignore_file_path: &Path,
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
            ignore_file_path,
            object_store,
            snapshot_store,
            snapshot,
            index,
        )?;
    }

    let (restored, skipped) = restore_files(project_root, snapshot, object_store, dry_run)?;

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
