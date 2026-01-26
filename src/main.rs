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
    let project_root = cli
        .project_root
        .clone()
        .unwrap_or_else(|| std::env::current_dir().expect("Failed to get current directory"));

    let allow_missing_project = matches!(
        &cli.command,
        Commands::Context {
            command: cli::ContextCommands::New { .. }
        } | Commands::Migrate { .. }
    );

    let resolve_opts = ResolveOptions {
        config_dir: cli.config_dir.clone(),
        project: cli.project.clone(),
        context: cli.context.clone(),
        project_root: project_root.clone(),
        allow_missing_project,
    };

    let config_resolver = ConfigResolver::load(&resolve_opts)?;
    let config = config_resolver.resolve();

    let ignore_file_path = cli
        .ignore_file
        .clone()
        .or_else(|| config_resolver.context_ignore_path())
        .unwrap_or_else(|| {
            resolve_ignore_file_path(&project_root, None, &config.ignore.ignore_file)
        });

    let ignore_file_path = if ignore_file_path.is_absolute() {
        ignore_file_path
    } else {
        project_root.join(ignore_file_path)
    };

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

    let ctx = CommandContext {
        project_root: &project_root,
        config: &config,
        storage_dir: resolved_storage_dir.as_deref(),
        ignore_file_path: ignore_file_path.clone(),
    };

    match cli.command {
        Commands::Init => commands::cmd_init(&ctx),
        Commands::Snapshot {
            message,
            trigger,
            auto,
        } => commands::cmd_snapshot(&ctx, message, trigger, auto),
        Commands::SetupShell { shell } => commands::cmd_setup_shell(&shell),
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
        Commands::Context { command } => commands::cmd_context(&config_resolver, command),
        Commands::Ignore { command } => commands::cmd_ignore(&ignore_file_path, command),
        Commands::Migrate { dry_run } => {
            commands::cmd_migrate(&project_root, &config_resolver, dry_run)
        }
    }
}
