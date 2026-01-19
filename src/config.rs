use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::error::{MoteError, Result};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum LocationStrategy {
    #[default]
    Root,
    Vcs,
    Auto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    #[serde(default)]
    pub location_strategy: LocationStrategy,
    #[serde(default = "default_compression_level")]
    pub compression_level: i32,
}

fn default_compression_level() -> i32 {
    3
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            location_strategy: LocationStrategy::default(),
            compression_level: default_compression_level(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotConfig {
    #[serde(default = "default_true")]
    pub auto_cleanup: bool,
    #[serde(default = "default_max_snapshots")]
    pub max_snapshots: u32,
    #[serde(default = "default_max_age_days")]
    pub max_age_days: u32,
}

fn default_true() -> bool {
    true
}

fn default_max_snapshots() -> u32 {
    1000
}

fn default_max_age_days() -> u32 {
    30
}

impl Default for SnapshotConfig {
    fn default() -> Self {
        Self {
            auto_cleanup: default_true(),
            max_snapshots: default_max_snapshots(),
            max_age_days: default_max_age_days(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgnoreConfig {
    #[serde(default = "default_ignore_file")]
    pub ignore_file: String,
}

fn default_ignore_file() -> String {
    ".moteignore".to_string()
}

impl Default for IgnoreConfig {
    fn default() -> Self {
        Self {
            ignore_file: default_ignore_file(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub storage: StorageConfig,
    #[serde(default)]
    pub snapshot: SnapshotConfig,
    #[serde(default)]
    pub ignore: IgnoreConfig,
}

impl Config {
    pub fn global_config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("mote").join("config.toml"))
    }

    pub fn load() -> Result<Self> {
        let config_path = match Self::global_config_path() {
            Some(p) => p,
            None => return Ok(Self::default()),
        };

        if !config_path.exists() {
            return Ok(Self::default());
        }

        let content =
            fs::read_to_string(&config_path).map_err(|e| MoteError::ConfigRead(e.to_string()))?;

        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save_default() -> Result<()> {
        let config_path = match Self::global_config_path() {
            Some(p) => p,
            None => return Ok(()),
        };

        if config_path.exists() {
            return Ok(());
        }

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let default_config = Self::default();
        let content = toml::to_string_pretty(&default_config)
            .map_err(|e| MoteError::ConfigParse(e.to_string()))?;

        fs::write(&config_path, content)?;
        Ok(())
    }
}
