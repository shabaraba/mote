mod context;
mod ignore;
mod init;
mod migrate;
mod snapshot;

use std::path::Path;

use crate::config::Config;
use crate::error::{MoteError, Result};
use crate::storage::StorageLocation;

pub use context::cmd_context;
pub use ignore::cmd_ignore;
pub use init::{cmd_init, cmd_setup_shell};
pub use migrate::cmd_migrate;
pub use snapshot::{cmd_delete, cmd_diff, cmd_gc, cmd_log, cmd_restore, cmd_show, cmd_snapshot};

pub struct CommandContext<'a> {
    pub project_root: &'a Path,
    pub config: &'a Config,
    pub storage_dir: Option<&'a Path>,
    pub ignore_file_path: std::path::PathBuf,
}

impl<'a> CommandContext<'a> {
    pub fn resolve_location(&self) -> Result<StorageLocation> {
        match StorageLocation::find_existing(self.project_root, self.storage_dir) {
            Ok(loc) => Ok(loc),
            Err(MoteError::NotInitialized) if self.storage_dir.is_some() => {
                StorageLocation::init(self.project_root, self.config, self.storage_dir)
            }
            Err(e) => Err(e),
        }
    }
}
