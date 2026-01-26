use std::path::Path;

use colored::*;

use crate::config::Config;
use crate::error::{MoteError, Result};
use crate::ignore::create_ignore_file;
use crate::storage::StorageLocation;

/// Context holding common parameters passed to command functions.
pub struct Context<'a> {
    /// The project root directory.
    pub project_root: &'a Path,
    /// The loaded configuration.
    pub config: &'a Config,
    /// Optional custom storage directory.
    pub storage_dir: Option<&'a Path>,
    /// Resolved ignore file path.
    pub ignore_file_path: std::path::PathBuf,
}

/// Initialize mote in the project directory.
/// Creates storage directories and default ignore file.
pub fn cmd_init(ctx: &Context) -> Result<()> {
    Config::save_default()?;
    let location = StorageLocation::init(ctx.project_root, ctx.config, ctx.storage_dir)?;

    // Create ignore file at resolved path
    let created_path = create_ignore_file(&ctx.ignore_file_path)?;

    // Display actual filename (show relative path if within project_root)
    let display_path = created_path
        .strip_prefix(ctx.project_root)
        .unwrap_or(&created_path);

    println!(
        "{} Initialized mote in {}",
        "✓".green().bold(),
        location.root().display()
    );
    println!(
        "  Created {} for ignore patterns",
        display_path.display().to_string().cyan()
    );
    Ok(())
}

/// Print shell integration script for auto-snapshot hooks.
/// Supports bash, zsh, and fish shells.
pub fn cmd_setup_shell(shell: &str) -> Result<()> {
    let script = match shell {
        "bash" | "zsh" => include_str!("../../scripts/shell_integration.sh"),
        "fish" => include_str!("../../scripts/shell_integration.fish"),
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
