use colored::*;

use crate::commands::CommandContext;
use crate::error::Result;
use crate::storage::{delete_objects, list_all_objects, ObjectReferences, SnapshotStore};

pub fn cmd_gc(ctx: &CommandContext, dry_run: bool, verbose: bool) -> Result<()> {
    let location = ctx.resolve_location()?;
    let snapshot_store = SnapshotStore::new(location.snapshots_dir());
    let objects_dir = location.objects_dir();

    if verbose {
        println!("{} Starting garbage collection...", "->".cyan().bold());
        println!("  Marking referenced objects...");
    }

    let snapshots = snapshot_store.list()?;
    let mut refs = ObjectReferences::new();
    for snapshot in &snapshots {
        refs.mark_from_snapshot(snapshot);
    }

    if verbose {
        println!(
            "  Found {} snapshots with {} unique objects",
            snapshots.len(),
            refs.referenced_count()
        );
        println!("  Scanning objects directory...");
    }

    let all_objects = list_all_objects(&objects_dir)?;
    let total_objects = all_objects.len();

    let unreferenced: Vec<String> = all_objects
        .into_iter()
        .filter(|hash| !refs.is_referenced(hash))
        .collect();

    if verbose {
        println!(
            "  Total objects: {}, Unreferenced: {}",
            total_objects,
            unreferenced.len()
        );
    }

    if unreferenced.is_empty() {
        println!("{} No unreferenced objects found", "✓".green().bold());
        return Ok(());
    }

    if dry_run {
        println!(
            "{} Would delete {} unreferenced object(s)",
            "dry-run".cyan().bold(),
            unreferenced.len()
        );
        if verbose {
            for hash in &unreferenced {
                println!("  Would delete: {}", hash.dimmed());
            }
        }
        return Ok(());
    }

    let stats = delete_objects(&objects_dir, &unreferenced, verbose)?;
    println!(
        "{} Deleted {} object(s), reclaimed {}",
        "✓".green().bold(),
        stats.deleted_objects,
        format_size(stats.deleted_bytes)
    );

    Ok(())
}

fn format_size(bytes: u64) -> String {
    let kb = bytes as f64 / 1024.0;
    if kb < 1024.0 {
        format!("{:.2} KB", kb)
    } else {
        format!("{:.2} MB", kb / 1024.0)
    }
}
