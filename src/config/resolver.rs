use std::path::{Path, PathBuf};

use crate::config::{Config, ContextConfig, ProjectConfig};
use crate::error::Result;

/// Options for resolving configuration from the 3-layer hierarchy
#[derive(Debug, Clone)]
pub struct ResolveOptions {
    /// Custom config directory (overrides default ~/.config/mote)
    pub config_dir: Option<PathBuf>,
    /// Explicit project name (if not specified, auto-detect from project_root)
    pub project: Option<String>,
    /// Context name within the project (defaults to "default")
    pub context: Option<String>,
    /// Current project root directory for auto-detection
    pub project_root: PathBuf,
    /// Allow missing project (for commands like context new that can create the project)
    pub allow_missing_project: bool,
}

/// Resolves configuration from the 3-layer hierarchy
///
/// This is the core component that implements the configuration resolution logic.
/// It loads and merges settings from:
/// 1. Global config (~/.config/mote/config.toml)
/// 2. Project config (~/.config/mote/projects/<name>/config.toml)
/// 3. Context config (~/.config/mote/projects/<name>/contexts/<context>/config.toml)
///
/// # Configuration Priority
///
/// Settings are applied in order, with later layers overriding earlier ones:
/// - **Global**: Base defaults for all projects
/// - **Project**: Project-specific overrides
/// - **Context**: Context-specific overrides (highest priority)
///
/// # Auto-Detection
///
/// If no explicit project is specified via CLI options, the resolver attempts
/// to auto-detect the project by matching the current directory against the
/// `cwd` paths stored in project configurations.
///
/// # Example
///
/// ```rust,no_run
/// use mote::config::{ConfigResolver, ResolveOptions};
/// use std::path::PathBuf;
///
/// let opts = ResolveOptions {
///     config_dir: None,  // Use default
///     project: Some("my-project".to_string()),
///     context: Some("feature-branch".to_string()),
///     project_root: PathBuf::from("/path/to/project"),
/// };
///
/// let resolver = ConfigResolver::load(&opts)?;
/// let config = resolver.resolve();
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct ConfigResolver {
    config_dir: PathBuf,
    global_config: Config,
    project_config: Option<ProjectConfig>,
    context_config: Option<ContextConfig>,
    project_name: Option<String>,
    context_name: String,
}

impl ConfigResolver {
    /// Load and resolve configuration based on options
    pub fn load(opts: &ResolveOptions) -> Result<Self> {
        // Determine config directory
        let config_dir = opts
            .config_dir
            .clone()
            .or_else(|| Config::global_config_path().map(|p| p.parent().unwrap().to_path_buf()))
            .unwrap_or_else(|| PathBuf::from(".config/mote"));

        // Load global config from the determined config_dir
        let global_config_path = config_dir.join("config.toml");
        let global_config = Config::load_from_path(&global_config_path)?;

        // Resolve project
        let (project_name, project_config) = if let Some(ref name) = opts.project {
            match ProjectConfig::load(&config_dir, name) {
                Ok(config) => (Some(name.clone()), Some(config)),
                Err(e) if opts.allow_missing_project => (Some(name.clone()), None),
                Err(e) => return Err(e),
            }
        } else {
            // Try to auto-detect from project_root
            if let Some(name) = ProjectConfig::find_by_path(&config_dir, &opts.project_root)? {
                let config = ProjectConfig::load(&config_dir, &name)?;
                (Some(name), Some(config))
            } else {
                (None, None)
            }
        };

        // Resolve context
        let context_name = opts.context.clone().unwrap_or_else(|| "default".to_string());

        let context_config = if let (Some(ref proj_name), Some(ref _proj_config)) =
            (&project_name, &project_config)
        {
            let project_dir = config_dir.join("projects").join(proj_name);

            // If context was explicitly specified, propagate errors
            // If using default context, allow it to be missing
            match ContextConfig::load(&project_dir, &context_name) {
                Ok(config) => Some(config),
                Err(e) => {
                    if opts.context.is_some() {
                        // Explicit context requested but failed to load - propagate error
                        return Err(e);
                    } else {
                        // Default context doesn't exist yet - that's ok
                        None
                    }
                }
            }
        } else {
            None
        };

        Ok(Self {
            config_dir,
            global_config,
            project_config,
            context_config,
            project_name,
            context_name,
        })
    }

    /// Resolve merged configuration (context > project > global)
    pub fn resolve(&self) -> Config {
        let mut result = self.global_config.clone();

        // Merge project config
        if let Some(ref project) = self.project_config {
            Self::merge_config(&mut result, &project.config);
        }

        // Merge context config
        if let Some(ref context) = self.context_config {
            Self::merge_config(&mut result, &context.config);
        }

        result
    }

    /// Get context storage directory (if context is configured)
    pub fn context_storage_dir(&self) -> Option<PathBuf> {
        if let (Some(ref project_name), Some(ref context)) =
            (&self.project_name, &self.context_config)
        {
            let project_dir = self.config_dir.join("projects").join(project_name);
            Some(context.storage_path(&project_dir, &self.context_name))
        } else {
            None
        }
    }

    /// Get context ignore file path (if context is configured)
    pub fn context_ignore_path(&self) -> Option<PathBuf> {
        if let (Some(ref project_name), Some(ref context)) =
            (&self.project_name, &self.context_config)
        {
            let project_dir = self.config_dir.join("projects").join(project_name);
            Some(context.ignore_path(&project_dir, &self.context_name))
        } else {
            None
        }
    }

    /// Get config directory
    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }

    /// Get project name (if resolved)
    pub fn project_name(&self) -> Option<&str> {
        self.project_name.as_deref()
    }

    /// Get context name
    #[allow(dead_code)]
    pub fn context_name(&self) -> &str {
        &self.context_name
    }

    /// Merge source config into target (source takes precedence)
    ///
    /// Since Config fields all have defaults, we perform a simple override:
    /// any non-default values in source replace the corresponding fields in target.
    ///
    /// Note: Current implementation does a full field replacement.
    /// For more granular control, individual config sections could be made Optional.
    fn merge_config(target: &mut Config, source: &Config) {
        // For storage config, override if source differs from default
        let default_storage = crate::config::StorageConfig::default();
        if source.storage.location_strategy != default_storage.location_strategy {
            target.storage.location_strategy = source.storage.location_strategy.clone();
        }

        // For snapshot config, override each field if different from default
        let default_snapshot = crate::config::SnapshotConfig::default();
        if source.snapshot.auto_cleanup != default_snapshot.auto_cleanup {
            target.snapshot.auto_cleanup = source.snapshot.auto_cleanup;
        }
        if source.snapshot.max_snapshots != default_snapshot.max_snapshots {
            target.snapshot.max_snapshots = source.snapshot.max_snapshots;
        }
        if source.snapshot.max_age_days != default_snapshot.max_age_days {
            target.snapshot.max_age_days = source.snapshot.max_age_days;
        }

        // For ignore config, override if different from default
        let default_ignore = crate::config::IgnoreConfig::default();
        if source.ignore.ignore_file != default_ignore.ignore_file {
            target.ignore.ignore_file = source.ignore.ignore_file.clone();
        }
    }
}
