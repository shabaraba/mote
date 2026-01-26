use std::path::Path;

use colored::*;

use crate::cli::IgnoreCommands;
use crate::error::Result;
use crate::ignore::create_ignore_file;

pub fn cmd_ignore(ignore_file_path: &Path, command: IgnoreCommands) -> Result<()> {
    match command {
        IgnoreCommands::List => {
            if !ignore_file_path.exists() {
                println!("{} No ignore file found", "!".yellow().bold());
                return Ok(());
            }

            let content = std::fs::read_to_string(ignore_file_path)?;
            println!("Ignore patterns in {}:", ignore_file_path.display());
            println!("{}", content);
        }
        IgnoreCommands::Add { pattern } => {
            let mut content = if ignore_file_path.exists() {
                std::fs::read_to_string(ignore_file_path)?
            } else {
                String::new()
            };

            if !content.ends_with('\n') && !content.is_empty() {
                content.push('\n');
            }
            content.push_str(&pattern);
            content.push('\n');

            std::fs::write(ignore_file_path, content)?;

            println!(
                "{} Added pattern '{}' to {}",
                "✓".green().bold(),
                pattern,
                ignore_file_path.display()
            );
        }
        IgnoreCommands::Remove { pattern } => {
            if !ignore_file_path.exists() {
                println!("{} No ignore file found", "!".yellow().bold());
                return Ok(());
            }

            let content = std::fs::read_to_string(ignore_file_path)?;
            let lines: Vec<&str> = content.lines().collect();
            let filtered: Vec<&str> = lines
                .into_iter()
                .filter(|line| line.trim() != pattern.trim())
                .collect();

            std::fs::write(ignore_file_path, filtered.join("\n") + "\n")?;

            println!(
                "{} Removed pattern '{}' from {}",
                "✓".green().bold(),
                pattern,
                ignore_file_path.display()
            );
        }
        IgnoreCommands::Edit => {
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());

            if !ignore_file_path.exists() {
                create_ignore_file(ignore_file_path)?;
            }

            let parts = shell_words::split(&editor).map_err(|e| {
                crate::error::MoteError::ConfigRead(format!("Failed to parse EDITOR: {}", e))
            })?;

            if parts.is_empty() {
                return Err(crate::error::MoteError::ConfigRead(
                    "EDITOR variable is empty".to_string(),
                ));
            }

            let status = std::process::Command::new(&parts[0])
                .args(&parts[1..])
                .arg(ignore_file_path)
                .status()?;

            if !status.success() {
                return Err(crate::error::MoteError::ConfigRead(format!(
                    "Editor '{}' exited with error",
                    editor
                )));
            }

            println!("{} Edited {}", "✓".green().bold(), ignore_file_path.display());
        }
    }

    Ok(())
}
