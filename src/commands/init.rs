use colored::*;

use super::CommandContext;
use crate::config::Config;
use crate::error::{MoteError, Result};
use crate::ignore::create_ignore_file;
use crate::storage::StorageLocation;

pub fn cmd_init(ctx: &CommandContext) -> Result<()> {
    Config::save_default()?;
    let location = StorageLocation::init(ctx.project_root, ctx.config, ctx.storage_dir)?;
    let created_path = create_ignore_file(&ctx.ignore_file_path)?;
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
