mod collect;
mod diff;
mod restore;

use colored::*;

use crate::commands::CommandContext;
use crate::error::{MoteError, Result};
use crate::storage::{Index, ObjectStore, Snapshot, SnapshotStore, StorageLocation};
use collect::{collect_files, have_same_file_hashes};

pub use diff::cmd_diff;
pub use restore::cmd_restore;

pub fn cmd_snapshot(
    ctx: &CommandContext,
    message: Option<String>,
    trigger: Option<String>,
    auto: bool,
) -> Result<()> {
    let location = match StorageLocation::find_existing(ctx.project_root, ctx.storage_dir) {
        Ok(loc) => loc,
        Err(MoteError::NotInitialized) if ctx.storage_dir.is_some() => {
            StorageLocation::init(ctx.project_root, ctx.config, ctx.storage_dir)?
        }
        Err(_) if auto => return Ok(()),
        Err(e) => return Err(e),
    };
    let object_store = ObjectStore::new(location.objects_dir());
    let snapshot_store = SnapshotStore::new(location.snapshots_dir());

    let mut index = Index::load(&location.index_path())?;
    let files = collect_files(
        ctx.project_root,
        &ctx.ignore_file_path,
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
            if let Some(latest) = snapshots.iter().max_by_key(|s| s.timestamp) {
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

pub fn cmd_log(ctx: &CommandContext, limit: usize, oneline: bool) -> Result<()> {
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

pub fn cmd_show(ctx: &CommandContext, snapshot_id: &str) -> Result<()> {
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
