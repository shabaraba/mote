mod cli;
mod commands;
mod config;
mod error;
mod ignore;
mod path_resolver;
mod storage;

use std::path::Path;

use clap::Parser;
use colored::*;

use cli::{Cli, Commands};
use config::{Config, ConfigResolver, ResolveOptions};
use error::Result;
use path_resolver::resolve_ignore_file_path;

/// Context holding common parameters passed to command functions.
struct Context<'a> {
    /// The project root directory.
    project_root: &'a Path,
    /// The loaded configuration.
    config: &'a Config,
    /// Optional custom storage directory.
    storage_dir: Option<&'a Path>,
    /// Resolved ignore file path.
    ignore_file_path: std::path::PathBuf,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{}: {}", "error".red().bold(), e);
        std::process::exit(1);
    }
}

/// Main entry point for command execution.
/// Parses CLI arguments, resolves paths, and dispatches to appropriate command handlers.
fn run() -> Result<()> {
    let cli = Cli::parse();
    let project_root = cli
        .project_root
        .clone()
        .unwrap_or_else(|| std::env::current_dir().expect("Failed to get current directory"));

    // Check if this is a context new command (allow missing project for auto-creation)
    let allow_missing_project = matches!(
        &cli.command,
        Commands::Context {
            command: cli::ContextCommands::New { .. }
        }
    );

    // Resolve configuration using ConfigResolver
    let resolve_opts = ResolveOptions {
        config_dir: cli.config_dir.clone(),
        project: cli.project.clone(),
        context: cli.context.clone(),
        project_root: project_root.clone(),
        allow_missing_project,
    };

    let config_resolver = ConfigResolver::load(&resolve_opts)?;
    let config = config_resolver.resolve();

    // Resolve ignore file path (CLI > Context > Config default)
    let ignore_file_path = cli
        .ignore_file
        .clone()
        .or_else(|| config_resolver.context_ignore_path())
        .unwrap_or_else(|| {
            resolve_ignore_file_path(&project_root, None, &config.ignore.ignore_file)
        });

    // Normalize ignore_file_path to absolute path
    let ignore_file_path = if ignore_file_path.is_absolute() {
        ignore_file_path
    } else {
        project_root.join(ignore_file_path)
    };

    // Resolve storage directory (CLI > Context > None)
    let resolved_storage_dir = cli
        .storage_dir
        .clone()
        .or_else(|| config_resolver.context_storage_dir())
        .map(|path| {
            if path.is_absolute() {
                path
            } else {
                project_root.join(path)
            }
        });

    let ctx = Context {
        project_root: &project_root,
        config: &config,
        storage_dir: resolved_storage_dir.as_deref(),
        ignore_file_path: ignore_file_path.clone(),
    };

    match cli.command {
        Commands::Init => {
            let init_ctx = commands::init::Context {
                project_root: ctx.project_root,
                config: ctx.config,
                storage_dir: ctx.storage_dir,
                ignore_file_path: ctx.ignore_file_path,
            };
            commands::cmd_init(&init_ctx)
        }
        Commands::Snapshot {
            message,
            trigger,
            auto,
        } => {
            let snapshot_ctx = commands::snapshot::Context {
                project_root: ctx.project_root,
                config: ctx.config,
                storage_dir: ctx.storage_dir,
                ignore_file_path: ctx.ignore_file_path,
            };
            commands::cmd_snapshot(&snapshot_ctx, message, trigger, auto)
        }
        Commands::SetupShell { shell } => commands::cmd_setup_shell(&shell),
        Commands::Log { limit, oneline } => {
            let snapshot_ctx = commands::snapshot::Context {
                project_root: ctx.project_root,
                config: ctx.config,
                storage_dir: ctx.storage_dir,
                ignore_file_path: ctx.ignore_file_path,
            };
            commands::cmd_log(&snapshot_ctx, limit, oneline)
        }
        Commands::Show { snapshot_id } => {
            let snapshot_ctx = commands::snapshot::Context {
                project_root: ctx.project_root,
                config: ctx.config,
                storage_dir: ctx.storage_dir,
                ignore_file_path: ctx.ignore_file_path,
            };
            commands::cmd_show(&snapshot_ctx, &snapshot_id)
        }
        Commands::Diff {
            snapshot_id,
            snapshot_id2,
            name_only,
            output,
            unified,
        } => {
            let snapshot_ctx = commands::snapshot::Context {
                project_root: ctx.project_root,
                config: ctx.config,
                storage_dir: ctx.storage_dir,
                ignore_file_path: ctx.ignore_file_path,
            };
            commands::cmd_diff(
                &snapshot_ctx,
                snapshot_id,
                snapshot_id2,
                name_only,
                output,
                unified,
            )
        }
        Commands::Restore {
            snapshot_id,
            file,
            force,
            dry_run,
        } => {
            let snapshot_ctx = commands::snapshot::Context {
                project_root: ctx.project_root,
                config: ctx.config,
                storage_dir: ctx.storage_dir,
                ignore_file_path: ctx.ignore_file_path,
            };
            commands::cmd_restore(&snapshot_ctx, &snapshot_id, file, force, dry_run)
        }
        Commands::Context { command } => commands::cmd_context(&config_resolver, command),
        Commands::Ignore { command } => commands::cmd_ignore(&ignore_file_path, command),
        Commands::Migrate { dry_run } => {
            commands::cmd_migrate(&project_root, &config_resolver, dry_run)
        }
    }
}
