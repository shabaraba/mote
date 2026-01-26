mod context;
mod ignore;
mod init;
mod migrate;
mod snapshot;

use std::path::Path;

use crate::config::Config;

pub use context::cmd_context;
pub use ignore::cmd_ignore;
pub use init::{cmd_init, cmd_setup_shell};
pub use migrate::cmd_migrate;
pub use snapshot::{cmd_diff, cmd_log, cmd_restore, cmd_show, cmd_snapshot};

pub struct CommandContext<'a> {
    pub project_root: &'a Path,
    pub config: &'a Config,
    pub storage_dir: Option<&'a Path>,
    pub ignore_file_path: std::path::PathBuf,
}
