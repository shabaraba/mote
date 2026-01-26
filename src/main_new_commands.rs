// New commands for project/context management

use std::path::{Path, PathBuf};
use colored::*;

use crate::config::{Config, ConfigResolver, ContextConfig, ProjectConfig};
use crate::error::Result;
use crate::ignore::create_ignore_file;
use crate::cli::{ContextCommands, IgnoreCommands};

/// Manage contexts
pub fn cmd_context(config_resolver: &ConfigResolver, command: ContextCommands) -> Result<()> {
    let config_dir = config_resolver.config_dir();
    let project_name = config_resolver
        .project_name()
        .ok_or_else(|| crate::error::MoteError::ConfigRead(
            "No project specified or detected. Use --project or run from project directory.".to_string()
        ))?;

    let project_dir = config_dir.join("projects").join(project_name);

    match command {
        ContextCommands::List => {
            let contexts = ContextConfig::list(&project_dir)?;
            if contexts.is_empty() {
                println!("{} No contexts found", "!".yellow().bold());
            } else {
                println!("Contexts for project '{}':", project_name);
                for ctx in contexts {
                    if ctx == "default" {
                        println!("  {} (default)", ctx.cyan());
                    } else {
                        println!("  {}", ctx.cyan());
                    }
                }
            }
        }
        ContextCommands::New { name, cwd, context_dir } => {
            // Load or create project config
            let mut project_config = if project_dir.exists() {
                ProjectConfig::load(config_dir, project_name)?
            } else {
                // Create new project
                let project_cwd = cwd.clone().unwrap_or_else(|| std::env::current_dir().expect("Failed to get current directory"));
                let config = ProjectConfig {
                    path: project_cwd.canonicalize().unwrap_or(project_cwd),
                    contexts: None,
                    config: Config::default(),
                };
                config.save(config_dir, project_name)?;

                println!(
                    "{} Created project '{}'",
                    "✓".green().bold(),
                    project_name
                );
                config
            };

            // Determine actual context directory
            let actual_context_dir = if let Some(custom_dir) = context_dir.clone() {
                // Register custom context directory in project config
                project_config.register_context(name.clone(), custom_dir.clone());
                project_config.save(config_dir, project_name)?;
                custom_dir
            } else {
                project_dir.join("contexts").join(&name)
            };

            let context_config = ContextConfig {
                cwd,
                context_dir,
                config: Config::default(),
            };

            context_config.save(&project_dir, &name)?;

            // Create ignore file
            let ignore_path = context_config.ignore_path(&actual_context_dir);
            create_ignore_file(&ignore_path)?;

            println!(
                "{} Created context '{}' for project '{}'",
                "✓".green().bold(),
                name,
                project_name
            );
            if context_config.context_dir.is_some() {
                println!(
                    "  Context directory: {}",
                    actual_context_dir.display().to_string().cyan()
                );
            }
        }
        ContextCommands::Delete { name } => {
            // Validate name before constructing path to prevent traversal attacks
            if name.is_empty() {
                return Err(crate::error::MoteError::InvalidName(
                    "Context name cannot be empty".to_string(),
                ));
            }

            // Reject path traversal attempts
            if name.contains("..") || name.contains('/') || name.contains('\\') {
                return Err(crate::error::MoteError::InvalidName(format!(
                    "Invalid context name: '{}'",
                    name
                )));
            }

            // Reject absolute paths
            if name.starts_with('/') || name.starts_with('\\') {
                return Err(crate::error::MoteError::InvalidName(format!(
                    "Context name cannot be absolute: '{}'",
                    name
                )));
            }

            if name == "default" {
                return Err(crate::error::MoteError::ConfigRead(
                    "Cannot delete default context".to_string(),
                ));
            }

            // Load project config to get context directory
            let mut project_config = ProjectConfig::load(config_dir, project_name)?;
            let context_dir = project_config.get_context_dir(&project_dir, &name);

            if !context_dir.exists() {
                return Err(crate::error::MoteError::ContextNotFound(name));
            }

            std::fs::remove_dir_all(&context_dir)?;

            // Unregister from project config if it was custom
            project_config.unregister_context(&name);
            project_config.save(config_dir, project_name)?;

            println!(
                "{} Deleted context '{}' from project '{}'",
                "✓".green().bold(),
                name,
                project_name
            );
        }
    }

    Ok(())
}

/// Manage ignore patterns
pub fn cmd_ignore(ctx: &crate::Context, command: IgnoreCommands) -> Result<()> {
    let ignore_path = &ctx.ignore_file_path;

    match command {
        IgnoreCommands::List => {
            if !ignore_path.exists() {
                println!("{} No ignore file found", "!".yellow().bold());
                return Ok(());
            }

            let content = std::fs::read_to_string(ignore_path)?;
            println!("Ignore patterns in {}:", ignore_path.display());
            println!("{}", content);
        }
        IgnoreCommands::Add { pattern } => {
            let mut content = if ignore_path.exists() {
                std::fs::read_to_string(ignore_path)?
            } else {
                String::new()
            };

            if !content.ends_with('\n') && !content.is_empty() {
                content.push('\n');
            }
            content.push_str(&pattern);
            content.push('\n');

            std::fs::write(ignore_path, content)?;

            println!(
                "{} Added pattern '{}' to {}",
                "✓".green().bold(),
                pattern,
                ignore_path.display()
            );
        }
        IgnoreCommands::Remove { pattern } => {
            if !ignore_path.exists() {
                println!("{} No ignore file found", "!".yellow().bold());
                return Ok(());
            }

            let content = std::fs::read_to_string(ignore_path)?;
            let lines: Vec<&str> = content.lines().collect();
            let filtered: Vec<&str> = lines
                .into_iter()
                .filter(|line| line.trim() != pattern.trim())
                .collect();

            std::fs::write(ignore_path, filtered.join("\n") + "\n")?;

            println!(
                "{} Removed pattern '{}' from {}",
                "✓".green().bold(),
                pattern,
                ignore_path.display()
            );
        }
        IgnoreCommands::Edit => {
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());

            if !ignore_path.exists() {
                create_ignore_file(ignore_path)?;
            }

            let status = std::process::Command::new(&editor)
                .arg(ignore_path)
                .status()?;

            if !status.success() {
                return Err(crate::error::MoteError::ConfigRead(format!(
                    "Editor '{}' exited with error",
                    editor
                )));
            }

            println!("{} Edited {}", "✓".green().bold(), ignore_path.display());
        }
    }

    Ok(())
}

/// Migrate existing .mote directory to new structure
pub fn cmd_migrate(
    project_root: &Path,
    config_resolver: &ConfigResolver,
    dry_run: bool,
) -> Result<()> {
    let old_mote_dir = project_root.join(".mote");

    if !old_mote_dir.exists() {
        println!("{} No .mote directory found to migrate", "!".yellow().bold());
        return Ok(());
    }

    // Detect project name from directory name
    let project_name = project_root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("migrated-project");

    println!("Migrating .mote/ to new structure...");
    println!("  Project name: {}", project_name.cyan());
    println!("  Source: {}", old_mote_dir.display());

    let config_dir = config_resolver.config_dir();
    let new_project_dir = config_dir.join("projects").join(project_name);
    let new_context_dir = new_project_dir.join("contexts").join("default");
    let new_storage_dir = new_context_dir.join("storage");

    println!("  Destination: {}", new_storage_dir.display());

    if dry_run {
        println!("\n{} Dry run - no changes made", "i".cyan().bold());
        return Ok(());
    }

    // Create project
    let project_config = ProjectConfig {
        path: project_root.canonicalize().unwrap_or_else(|_| project_root.to_path_buf()),
        contexts: None,
        config: Config::default(),
    };
    project_config.save(config_dir, project_name)?;

    // Create context
    let context_config = ContextConfig {
        cwd: Some(project_root.to_path_buf()),
        context_dir: None,
        config: Config::default(),
    };
    context_config.save(&new_project_dir, "default")?;

    // Move .mote contents to new location
    for entry in std::fs::read_dir(&old_mote_dir)? {
        let entry = entry?;
        let dest = new_storage_dir.join(entry.file_name());
        if entry.path().is_dir() {
            copy_dir_all(&entry.path(), &dest)?;
        } else {
            std::fs::create_dir_all(&new_storage_dir)?;
            std::fs::copy(&entry.path(), &dest)?;
        }
    }

    // Create ignore file
    let old_ignore = project_root.join(".moteignore");
    let new_ignore = new_context_dir.join("ignore");

    if old_ignore.exists() {
        std::fs::copy(&old_ignore, &new_ignore)?;
        println!("  Copied .moteignore to context");
    } else {
        create_ignore_file(&new_ignore)?;
    }

    println!(
        "\n{} Migration complete!",
        "✓".green().bold()
    );
    println!("  You can now remove the old .mote/ directory");
    println!("  Use: -p {} -c default for future commands", project_name);

    Ok(())
}

/// Recursively copy directory contents with security checks
///
/// # Security features:
/// - Symlinks are skipped (not followed) to prevent path traversal
/// - Validates that destination is not a subdirectory of source
/// - Only copies regular files and directories
fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    // Normalize paths to prevent traversal attacks
    let src_canonical = src.canonicalize()?;

    // Create destination directory if it doesn't exist
    std::fs::create_dir_all(dst)?;

    // Check if destination would be inside source (prevent infinite loop)
    if let Ok(dst_canonical) = dst.canonicalize() {
        if dst_canonical.starts_with(&src_canonical) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Destination cannot be a subdirectory of source",
            ));
        }
    }

    copy_dir_all_impl(&src_canonical, dst)
}

fn copy_dir_all_impl(src: &Path, dst: &Path) -> std::io::Result<()> {
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        // Get metadata without following symlinks
        let metadata = entry.metadata()?;

        // Skip symlinks for security (don't follow them)
        if metadata.is_symlink() {
            eprintln!("Warning: Skipping symbolic link: {:?}", src_path);
            continue;
        }

        if metadata.is_dir() {
            std::fs::create_dir_all(&dst_path)?;
            copy_dir_all_impl(&src_path, &dst_path)?;
        } else if metadata.is_file() {
            std::fs::copy(&src_path, &dst_path)?;
        }
        // Ignore other file types (devices, sockets, etc.)
    }
    Ok(())
}
