mod cli;
mod commands;
mod config;
mod error;
mod ignore;
mod path_resolver;
mod storage;

use clap::Parser;
use colored::*;

use cli::{Cli, Commands};
use commands::CommandContext;
use config::{ConfigResolver, ResolveOptions};
use error::Result;
use path_resolver::resolve_ignore_file_path;

fn main() {
    if let Err(e) = run() {
        eprintln!("{}: {}", "error".red().bold(), e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    // Parse context specifier and validate options
    let (project, context) = cli.parse_context_spec()?;

    let project_root = cli
        .project_root
        .clone()
        .unwrap_or_else(|| std::env::current_dir().expect("Failed to get current directory"));

    // Standalone mode detection: --context-dir without -c/--context
    let is_standalone_mode = cli.context_dir.is_some()
        && !matches!(&cli.command, Commands::Context { .. });

    let allow_missing_project = matches!(
        &cli.command,
        Commands::Context {
            command: cli::ContextCommands::New { .. }
        } | Commands::Migrate { .. }
    ) || is_standalone_mode;

    let resolve_opts = ResolveOptions {
        config_dir: cli.config_dir.clone(),
        project,
        context,
        context_dir: cli.context_dir.clone(),
        project_root: project_root.clone(),
        allow_missing_project,
    };

    let config_resolver = ConfigResolver::load(&resolve_opts)?;
    let config = config_resolver.resolve();

    // Auto-initialize context directory if in standalone mode
    if let Some(ref ctx_dir) = cli.context_dir {
        if is_standalone_mode {
            if !ctx_dir.exists() {
                std::fs::create_dir_all(ctx_dir)?;
                std::fs::create_dir_all(ctx_dir.join("storage"))?;
                std::fs::create_dir_all(ctx_dir.join("storage/objects"))?;
                std::fs::create_dir_all(ctx_dir.join("storage/snapshots"))?;

                // Create default ignore file
                let ignore_path = ctx_dir.join("ignore");
                if !ignore_path.exists() {
                    crate::ignore::create_ignore_file(&ignore_path)?;
                }
            }
        }
    }

    let ignore_file_path = if is_standalone_mode {
        // Standalone mode: use context_dir/ignore
        cli.context_dir.as_ref().unwrap().join("ignore")
    } else {
        // Normal mode: use context ignore path or project default
        config_resolver
            .context_ignore_path()
            .unwrap_or_else(|| {
                resolve_ignore_file_path(&project_root, None, &config.ignore.ignore_file)
            })
    };

    let ignore_file_path = if ignore_file_path.is_absolute() {
        ignore_file_path
    } else {
        project_root.join(ignore_file_path)
    };

    let resolved_storage_dir = if is_standalone_mode {
        // Standalone mode: use context_dir/storage
        Some(cli.context_dir.as_ref().unwrap().join("storage"))
    } else {
        // Normal mode: use context storage
        config_resolver.context_storage_dir().map(|path| {
            if path.is_absolute() {
                path
            } else {
                project_root.join(path)
            }
        })
    };

    let ctx = CommandContext {
        project_root: &project_root,
        config: &config,
        storage_dir: resolved_storage_dir.as_deref(),
        ignore_file_path: ignore_file_path.clone(),
    };

    match cli.command {
        Commands::Snap { command } => match command {
            None | Some(cli::SnapCommands::Create { .. }) => {
                let (message, trigger, auto) = if let Some(cli::SnapCommands::Create {
                    message,
                    trigger,
                    auto,
                }) = command
                {
                    (message, trigger, auto)
                } else {
                    (None, None, false)
                };
                commands::cmd_snapshot(&ctx, message, trigger, auto)
            }
            Some(cli::SnapCommands::List { limit, oneline }) => {
                commands::cmd_log(&ctx, limit, oneline)
            }
            Some(cli::SnapCommands::Show { snapshot_id }) => {
                commands::cmd_show(&ctx, &snapshot_id)
            }
            Some(cli::SnapCommands::Diff {
                snapshot_id,
                snapshot_id2,
                name_only,
                output,
                unified,
            }) => commands::cmd_diff(&ctx, snapshot_id, snapshot_id2, name_only, output, unified),
            Some(cli::SnapCommands::Restore {
                snapshot_id,
                file,
                force,
                dry_run,
            }) => commands::cmd_restore(&ctx, &snapshot_id, file, force, dry_run),
        },
        Commands::Project { command } => match command {
            cli::ProjectCommands::List => {
                let config_dir = config_resolver.config_dir();
                let projects = crate::config::ProjectConfig::list(config_dir)?;
                if projects.is_empty() {
                    println!("No projects found.");
                } else {
                    for project in projects {
                        println!("{}", project);
                    }
                }
                Ok(())
            }
            cli::ProjectCommands::Init { name: _ } => {
                // TODO: Implement proper project init with custom name
                commands::cmd_init(&ctx)
            }
        },
        Commands::Context { command } => {
            commands::cmd_context(&config_resolver, command, cli.context_dir.as_ref())
        }
        Commands::Ignore { command } => commands::cmd_ignore(&ignore_file_path, command),
        Commands::Setup { shell } => commands::cmd_setup_shell(&shell),
        Commands::Migrate { dry_run } => {
            commands::cmd_migrate(&project_root, &config_resolver, dry_run)
        }
        // Backward compatibility aliases
        Commands::Snapshot {
            message,
            trigger,
            auto,
        } => commands::cmd_snapshot(&ctx, message, trigger, auto),
        Commands::Log { limit, oneline } => commands::cmd_log(&ctx, limit, oneline),
        Commands::Show { snapshot_id } => commands::cmd_show(&ctx, &snapshot_id),
        Commands::Diff {
            snapshot_id,
            snapshot_id2,
            name_only,
            output,
            unified,
        } => commands::cmd_diff(&ctx, snapshot_id, snapshot_id2, name_only, output, unified),
        Commands::Restore {
            snapshot_id,
            file,
            force,
            dry_run,
        } => commands::cmd_restore(&ctx, &snapshot_id, file, force, dry_run),
        Commands::SetupShell { shell } => commands::cmd_setup_shell(&shell),
        Commands::Init => commands::cmd_init(&ctx),
    }
}
