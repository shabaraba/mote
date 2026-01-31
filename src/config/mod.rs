//! Configuration management for Mote
//!
//! This module implements a **3-layer configuration hierarchy** to provide flexible
//! snapshot management across different projects and contexts.
//!
//! # Configuration Architecture
//!
//! ## 3-Layer Hierarchy
//!
//! Configuration is resolved through three layers, with later layers overriding earlier ones:
//!
//! 1. **Global Layer** (`~/.config/mote/config.toml`)
//!    - User-wide default settings
//!    - Applies to all projects unless overridden
//!
//! 2. **Project Layer** (`~/.config/mote/projects/<name>/config.toml`)
//!    - Project-specific settings
//!    - Tied to a specific codebase via `cwd` path
//!    - Overrides global settings
//!
//! 3. **Context Layer** (`~/.config/mote/projects/<name>/contexts/<context>/config.toml`)
//!    - Named configuration profiles within a project
//!    - Useful for different branches, experiments, or workflows
//!    - Has highest priority, overrides project and global settings
//!
//! ## Directory Structure
//!
//! ```text
//! ~/.config/mote/
//! ├── config.toml              # Global configuration
//! ├── ignore                   # Global ignore patterns
//! └── projects/
//!     └── <project-name>/
//!         ├── config.toml      # Project configuration (stores cwd)
//!         └── contexts/
//!             └── <context-name>/
//!                 ├── config.toml   # Context-specific config
//!                 ├── ignore        # Context-specific ignore patterns
//!                 └── storage/      # Snapshot storage
//!                     ├── index
//!                     ├── objects/
//!                     └── snapshots/
//! ```
//!
//! ## Configuration Resolution
//!
//! The [`ConfigResolver`] merges settings from all three layers:
//!
//! ```text
//! Final Config = Global ← Project ← Context
//!                (base)    (override)  (highest priority)
//! ```
//!
//! ## Usage Examples
//!
//! ### Basic Usage
//!
//! ```bash
//! # Create a new context (this also creates the project if it doesn't exist)
//! mote context new default
//!
//! # Create additional contexts for different workflows
//! mote context new feature-branch
//! mote context new experiment
//!
//! # Use a specific context
//! mote -c feature-branch snapshot
//! mote -c experiment snapshot
//! ```
//!
//! ### Auto-Detection
//!
//! If you run `mote` commands from within a project directory, the project
//! is automatically detected via the `cwd` recorded in project configs:
//!
//! ```bash
//! cd /path/to/my/project
//! mote snapshot              # Auto-detects project from cwd
//! mote -c experiment snapshot  # Auto-detects project, uses experiment context
//! ```
//!
//! ## VCS Independence
//!
//! All configuration and snapshot data is stored in `~/.config/mote/`, keeping
//! project directories clean and free of `.mote/` or `.moteignore` files. This
//! ensures true VCS independence - no mote-specific files need to be added to
//! `.gitignore` or similar.

mod context;
mod project;
mod resolver;

#[cfg(test)]
mod tests;

pub use context::ContextConfig;
pub use project::ProjectConfig;
pub use resolver::{ConfigResolver, ResolveOptions};

// Re-export existing Config types
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
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            location_strategy: LocationStrategy::default(),
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
    #[serde(default)]
    pub gc_auto_enabled: bool,
    #[serde(default = "default_gc_auto")]
    pub gc_auto: usize,
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

fn default_gc_auto() -> usize {
    100
}

impl Default for SnapshotConfig {
    fn default() -> Self {
        Self {
            auto_cleanup: default_true(),
            max_snapshots: default_max_snapshots(),
            max_age_days: default_max_age_days(),
            gc_auto_enabled: false,
            gc_auto: default_gc_auto(),
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

    /// Load global configuration from default path
    ///
    /// Note: Prefer using `load_from_path` with an explicit config_dir
    #[allow(dead_code)]
    pub fn load() -> Result<Self> {
        let config_path = match Self::global_config_path() {
            Some(p) => p,
            None => return Ok(Self::default()),
        };

        Self::load_from_path(&config_path)
    }

    /// Load configuration from a specific path
    pub fn load_from_path(config_path: &std::path::Path) -> Result<Self> {
        if !config_path.exists() {
            return Ok(Self::default());
        }

        let content =
            fs::read_to_string(config_path).map_err(|e| MoteError::ConfigRead(e.to_string()))?;

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
