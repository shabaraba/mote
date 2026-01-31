use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::error::{MoteError, Result};

#[derive(Parser)]
#[command(name = "mote")]
#[command(author, version, about = "A fine-grained snapshot management tool", long_about = None)]
pub struct Cli {
    /// Context specifier: [project/]context
    /// Examples: myproject/feature, feature, myproject
    #[arg(short = 'c', long = "context", global = true)]
    pub context_spec: Option<String>,

    /// Context directory for standalone mode (no project management)
    #[arg(short = 'd', long = "context-dir", global = true)]
    pub context_dir: Option<PathBuf>,

    /// Custom project root (defaults to current directory)
    #[arg(long, global = true)]
    pub project_root: Option<PathBuf>,

    /// Custom config directory (overrides default ~/.config/mote)
    #[arg(long, global = true)]
    pub config_dir: Option<PathBuf>,

    // Deprecated options (hidden, for backward compatibility)
    #[arg(short = 'p', long, global = true, hide = true)]
    pub project: Option<String>,

    #[arg(long = "old-context", global = true, hide = true)]
    pub old_context: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Snapshot operations
    Snap {
        #[command(subcommand)]
        command: Option<SnapCommands>,
    },

    /// Project management
    Project {
        #[command(subcommand)]
        command: ProjectCommands,
    },

    /// Manage contexts
    Context {
        #[command(subcommand)]
        command: ContextCommands,
    },

    /// Manage ignore patterns
    Ignore {
        #[command(subcommand)]
        command: IgnoreCommands,
    },

    /// Print shell integration script
    Setup {
        /// Shell type (bash, zsh, fish)
        #[arg(default_value = "zsh")]
        shell: String,
    },

    /// Migrate existing .mote directory to new structure
    Migrate {
        /// Show what would be migrated without actually migrating
        #[arg(long)]
        dry_run: bool,
    },

    // Backward compatibility aliases (hidden)
    #[command(hide = true)]
    Snapshot {
        #[arg(short, long)]
        message: Option<String>,
        #[arg(short, long)]
        trigger: Option<String>,
        #[arg(long)]
        auto: bool,
    },

    #[command(hide = true)]
    Log {
        #[arg(short, long, default_value = "20")]
        limit: usize,
        #[arg(long)]
        oneline: bool,
    },

    #[command(hide = true)]
    Show {
        snapshot_id: String,
    },

    #[command(hide = true)]
    Diff {
        snapshot_id: Option<String>,
        snapshot_id2: Option<String>,
        #[arg(long)]
        name_only: bool,
        #[arg(short, long)]
        output: Option<String>,
        #[arg(short = 'U', long, default_value = "3")]
        unified: usize,
    },

    #[command(hide = true)]
    Restore {
        snapshot_id: String,
        #[arg(short, long)]
        file: Option<String>,
        #[arg(long)]
        force: bool,
        #[arg(long)]
        dry_run: bool,
    },

    #[command(hide = true)]
    SetupShell {
        #[arg(default_value = "zsh")]
        shell: String,
    },

    #[command(hide = true)]
    Init,
}

#[derive(Subcommand)]
pub enum SnapCommands {
    /// Create a new snapshot (default if no subcommand)
    Create {
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

    /// Show snapshot history
    List {
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

    /// Delete a snapshot
    Delete {
        /// Snapshot ID to delete
        snapshot_id: String,

        /// Skip confirmation prompt
        #[arg(long)]
        force: bool,
    },

    /// Run garbage collection to remove unreferenced objects
    Gc {
        /// Show what would be removed without actually removing
        #[arg(long)]
        dry_run: bool,

        /// Show detailed progress information
        #[arg(long)]
        verbose: bool,
    },
}

#[derive(Subcommand)]
pub enum ProjectCommands {
    /// List all projects
    List,

    /// Initialize a new project
    Init {
        /// Project name (defaults to current directory name)
        name: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum ContextCommands {
    /// List all contexts
    List,

    /// Create a new context
    New {
        /// Context name
        name: String,

        /// Working directory for this context
        #[arg(long)]
        cwd: Option<PathBuf>,

        /// Do not register this context in project config (for temporary contexts)
        #[arg(long)]
        no_register: bool,
    },

    /// Delete a context
    Delete {
        /// Context name
        name: String,
    },
}

#[derive(Subcommand)]
pub enum IgnoreCommands {
    /// List ignore patterns
    List,

    /// Add ignore pattern
    Add {
        /// Pattern to add
        pattern: String,
    },

    /// Remove ignore pattern
    Remove {
        /// Pattern to remove
        pattern: String,
    },

    /// Edit ignore file in editor
    Edit,
}

impl Cli {
    /// Parse context specifier into (project, context) tuple
    /// Examples:
    /// - "myproject/feature" -> (Some("myproject"), Some("feature"))
    /// - "feature" -> (None, Some("feature"))
    /// - "myproject" -> (Some("myproject"), None)
    pub fn parse_context_spec(&self) -> Result<(Option<String>, Option<String>)> {
        // Validate exclusivity
        if self.context_dir.is_some() {
            if self.context_spec.is_some() {
                return Err(MoteError::InvalidArguments(
                    "-d/--context-dir cannot be used with -c/--context".to_string(),
                ));
            }
            if self.config_dir.is_some() {
                return Err(MoteError::InvalidArguments(
                    "-d/--context-dir cannot be used with --config-dir".to_string(),
                ));
            }
            if self.project.is_some() {
                return Err(MoteError::InvalidArguments(
                    "-d/--context-dir cannot be used with -p/--project".to_string(),
                ));
            }
            if self.old_context.is_some() {
                return Err(MoteError::InvalidArguments(
                    "-d/--context-dir cannot be used with --old-context".to_string(),
                ));
            }
        }

        // Parse context_spec
        if let Some(ref spec) = self.context_spec {
            if let Some(pos) = spec.find('/') {
                // Check for multiple slashes
                if spec.rfind('/') != Some(pos) {
                    return Err(MoteError::InvalidArguments(
                        "Invalid context specifier format. Use [project/]context".to_string(),
                    ));
                }

                let project = spec[..pos].to_string();
                let context = spec[pos + 1..].to_string();

                if project.is_empty() || context.is_empty() {
                    return Err(MoteError::InvalidArguments(
                        "Invalid context specifier format. Use [project/]context".to_string(),
                    ));
                }

                Ok((Some(project), Some(context)))
            } else {
                // No slash: could be project or context
                // We'll treat it as context if it looks like a context name,
                // otherwise as project. For now, always treat as context.
                Ok((None, Some(spec.clone())))
            }
        } else if self.project.is_some() || self.old_context.is_some() {
            // Backward compatibility
            Ok((self.project.clone(), self.old_context.clone()))
        } else {
            Ok((None, None))
        }
    }
}
