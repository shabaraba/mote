use std::io::{self, Write};

use colored::*;

use crate::commands::CommandContext;
use crate::error::Result;
use crate::storage::SnapshotStore;

pub fn cmd_delete(ctx: &CommandContext, snapshot_id: &str, force: bool) -> Result<()> {
    let location = ctx.resolve_location()?;
    let snapshot_store = SnapshotStore::new(location.snapshots_dir());
    let snapshot = snapshot_store.find_by_id(snapshot_id)?;

    if !force {
        print!(
            "Delete snapshot {} ({} files)? [y/N] ",
            snapshot.short_id().cyan(),
            snapshot.file_count()
        );
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let answer = input.trim().to_lowercase();
        if answer != "y" && answer != "yes" {
            println!("{} Deletion cancelled", "!".yellow().bold());
            return Ok(());
        }
    }

    snapshot_store.delete(&snapshot.id)?;

    println!(
        "{} Deleted snapshot {} ({} files)",
        "✓".green().bold(),
        snapshot.short_id().cyan(),
        snapshot.file_count()
    );

    Ok(())
}
