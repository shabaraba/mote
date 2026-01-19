use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "mote")]
#[command(author, version, about = "A fine-grained snapshot management tool", long_about = None)]
pub struct Cli {
    /// Custom project root (defaults to current directory)
    #[arg(long)]
    pub project_root: Option<PathBuf>,

    /// Custom ignore file path (overrides config)
    #[arg(long)]
    pub ignore_file: Option<PathBuf>,

    /// Custom storage directory
    #[arg(long)]
    pub storage_dir: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize mote in the current directory
    Init,

    /// Create a new snapshot
    Snapshot {
        /// Optional message for the snapshot
        #[arg(short, long)]
        message: Option<String>,

        /// Trigger source (e.g., "claude-code-hook", "manual")
        #[arg(short, long)]
        trigger: Option<String>,

        /// Auto mode: skip if no changes, quiet output (for git/jj hooks)
        #[arg(long)]
        auto: bool,
    },

    /// Print shell integration script for git/jj auto-snapshot
    SetupShell {
        /// Shell type (bash, zsh, fish)
        #[arg(default_value = "zsh")]
        shell: String,
    },

    /// Show snapshot history
    Log {
        /// Maximum number of snapshots to show
        #[arg(short, long, default_value = "20")]
        limit: usize,

        /// Show compact one-line format
        #[arg(long)]
        oneline: bool,
    },

    /// Show details of a specific snapshot
    Show {
        /// Snapshot ID (can be abbreviated)
        snapshot_id: String,
    },

    /// Show differences between snapshots or working directory
    Diff {
        /// First snapshot ID (if omitted, uses latest snapshot)
        snapshot_id: Option<String>,

        /// Second snapshot ID (optional, compares with current working directory if omitted)
        snapshot_id2: Option<String>,

        /// Show only file names without diff content
        #[arg(long)]
        name_only: bool,

        /// Output diff to a file (.diff or .patch)
        #[arg(short, long)]
        output: Option<String>,

        /// Number of context lines (default: 3)
        #[arg(short = 'U', long, default_value = "3")]
        unified: usize,
    },

    /// Restore files from a snapshot
    Restore {
        /// Snapshot ID to restore from
        snapshot_id: String,

        /// Specific file to restore (restores entire snapshot if omitted)
        #[arg(short, long)]
        file: Option<String>,

        /// Skip automatic backup creation before restore
        #[arg(long)]
        force: bool,

        /// Show what would be restored without actually restoring
        #[arg(long)]
        dry_run: bool,
    },
}
