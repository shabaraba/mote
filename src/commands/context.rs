use colored::*;

use crate::cli::ContextCommands;
use crate::config::{Config, ConfigResolver, ContextConfig, ProjectConfig};
use crate::error::Result;
use crate::ignore::create_ignore_file;

pub fn cmd_context(config_resolver: &ConfigResolver, command: ContextCommands) -> Result<()> {
    let config_dir = config_resolver.config_dir();
    let project_name = config_resolver.project_name().ok_or_else(|| {
        crate::error::MoteError::ConfigRead(
            "No project specified or detected. Use --project or run from project directory."
                .to_string(),
        )
    })?;

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
        ContextCommands::New {
            name,
            cwd,
            context_dir,
        } => {
            let mut project_config = if project_dir.exists() {
                ProjectConfig::load(config_dir, project_name)?
            } else {
                let project_cwd = cwd.clone().unwrap_or_else(|| {
                    std::env::current_dir().expect("Failed to get current directory")
                });
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

            let actual_context_dir = if let Some(custom_dir) = context_dir.clone() {
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
            validate_context_name(&name)?;

            if name == "default" {
                return Err(crate::error::MoteError::ConfigRead(
                    "Cannot delete default context".to_string(),
                ));
            }

            let mut project_config = ProjectConfig::load(config_dir, project_name)?;
            let context_dir = project_config.get_context_dir(&project_dir, &name);

            if !context_dir.exists() {
                return Err(crate::error::MoteError::ContextNotFound(name));
            }

            std::fs::remove_dir_all(&context_dir)?;

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

fn validate_context_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(crate::error::MoteError::InvalidName(
            "Context name cannot be empty".to_string(),
        ));
    }

    if name.contains("..") || name.contains('/') || name.contains('\\') {
        return Err(crate::error::MoteError::InvalidName(format!(
            "Invalid context name: '{}'",
            name
        )));
    }

    Ok(())
}
