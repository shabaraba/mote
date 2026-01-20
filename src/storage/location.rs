use std::path::{Path, PathBuf};

use crate::config::{Config, LocationStrategy};
use crate::error::{MoteError, Result};

pub struct StorageLocation {
    root: PathBuf,
}

impl StorageLocation {
    pub fn init(
        project_root: &Path,
        config: &Config,
        custom_storage_dir: Option<&Path>,
    ) -> Result<Self> {
        let storage_root = if let Some(custom_dir) = custom_storage_dir {
            custom_dir.to_path_buf()
        } else {
            Self::determine_storage_path(project_root, &config.storage.location_strategy)?
        };

        if storage_root.exists() {
            return Err(MoteError::AlreadyInitialized);
        }

        std::fs::create_dir_all(&storage_root)?;
        std::fs::create_dir_all(storage_root.join("objects"))?;
        std::fs::create_dir_all(storage_root.join("snapshots"))?;

        Ok(Self { root: storage_root })
    }

    fn determine_storage_path(project_root: &Path, strategy: &LocationStrategy) -> Result<PathBuf> {
        match strategy {
            LocationStrategy::Root => Ok(project_root.join(".mote")),
            LocationStrategy::Vcs => {
                if let Some(vcs_path) = Self::find_vcs_dir(project_root) {
                    Ok(vcs_path.join("mote"))
                } else {
                    Err(MoteError::NoVcsDirectory)
                }
            }
            LocationStrategy::Auto => {
                if let Some(vcs_path) = Self::find_vcs_dir(project_root) {
                    Ok(vcs_path.join("mote"))
                } else {
                    Ok(project_root.join(".mote"))
                }
            }
        }
    }

    fn find_vcs_dir(project_root: &Path) -> Option<PathBuf> {
        let git_dir = project_root.join(".git");
        if git_dir.is_dir() {
            return Some(git_dir);
        }

        let jj_dir = project_root.join(".jj");
        if jj_dir.is_dir() {
            return Some(jj_dir);
        }

        None
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn objects_dir(&self) -> PathBuf {
        self.root.join("objects")
    }

    pub fn snapshots_dir(&self) -> PathBuf {
        self.root.join("snapshots")
    }

    pub fn index_path(&self) -> PathBuf {
        self.root.join("index")
    }

    pub fn find_existing(project_root: &Path, custom_storage_dir: Option<&Path>) -> Result<Self> {
        if let Some(custom_dir) = custom_storage_dir {
            if custom_dir.exists() {
                return Ok(Self {
                    root: custom_dir.to_path_buf(),
                });
            } else {
                return Err(MoteError::NotInitialized);
            }
        }

        let mote_dir = project_root.join(".mote");
        if mote_dir.exists() {
            return Ok(Self { root: mote_dir });
        }

        let git_mote = project_root.join(".git").join("mote");
        if git_mote.exists() {
            return Ok(Self { root: git_mote });
        }

        let jj_mote = project_root.join(".jj").join("mote");
        if jj_mote.exists() {
            return Ok(Self { root: jj_mote });
        }

        Err(MoteError::NotInitialized)
    }
}
