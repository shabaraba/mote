use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::error::{MoteError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<PathBuf>,
    /// Custom context directory (if specified, context is stored here instead of default location)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_dir: Option<PathBuf>,
    #[serde(flatten)]
    pub config: Config,
}

impl ContextConfig {
    /// Load context configuration
    /// context_dir_override: Custom context directory (from ProjectConfig.contexts map)
    pub fn load(project_dir: &Path, context_name: &str, context_dir_override: Option<&PathBuf>) -> Result<Self> {
        Self::validate_name(context_name)?;

        let context_dir = if let Some(custom_dir) = context_dir_override {
            custom_dir.clone()
        } else {
            project_dir.join("contexts").join(context_name)
        };

        let config_path = context_dir.join("config.toml");

        if !config_path.exists() {
            return Err(MoteError::ContextNotFound(context_name.to_string()));
        }

        let content =
            fs::read_to_string(&config_path).map_err(|e| MoteError::ConfigRead(e.to_string()))?;

        let config: ContextConfig = toml::from_str(&content)?;
        Ok(config)
    }

    /// Save context configuration
    /// If context_dir is specified in the config, saves to that location
    /// Otherwise saves to default location: project_dir/contexts/<name>
    pub fn save(&self, project_dir: &Path, context_name: &str) -> Result<()> {
        Self::validate_name(context_name)?;

        // Determine where to save the context
        let context_dir = if let Some(ref custom_dir) = self.context_dir {
            custom_dir.clone()
        } else {
            project_dir.join("contexts").join(context_name)
        };

        if context_dir.exists() {
            let config_path = context_dir.join("config.toml");
            if config_path.exists() {
                return Err(MoteError::ContextAlreadyExists(context_name.to_string()));
            }
        }

        fs::create_dir_all(&context_dir)?;

        let config_path = context_dir.join("config.toml");
        let content = toml::to_string_pretty(self)
            .map_err(|e| MoteError::ConfigParse(e.to_string()))?;

        fs::write(&config_path, content)?;

        // Create storage directory
        let storage_dir = context_dir.join("storage");
        fs::create_dir_all(&storage_dir)?;
        fs::create_dir_all(storage_dir.join("objects"))?;
        fs::create_dir_all(storage_dir.join("snapshots"))?;

        Ok(())
    }

    /// List all context names for a project
    pub fn list(project_dir: &Path) -> Result<Vec<String>> {
        let contexts_dir = project_dir.join("contexts");

        if !contexts_dir.exists() {
            return Ok(Vec::new());
        }

        let mut names = Vec::new();

        for entry in fs::read_dir(contexts_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    names.push(name.to_string());
                }
            }
        }

        Ok(names)
    }

    /// Get storage directory path for this context
    /// Storage is always at context_dir/storage/
    pub fn storage_path(&self, context_dir: &Path) -> PathBuf {
        context_dir.join("storage")
    }

    /// Get ignore file path for this context
    pub fn ignore_path(&self, context_dir: &Path) -> PathBuf {
        context_dir.join("ignore")
    }

    /// Validate context name with comprehensive security checks
    ///
    /// # Security checks:
    /// - Length: 1-255 characters
    /// - First character: alphanumeric or underscore
    /// - Allowed characters: alphanumeric, hyphen, underscore
    /// - Path traversal prevention: no "..", "/", "\"
    /// - Control characters: not allowed
    /// - Reserved words: Windows reserved names blocked
    fn validate_name(name: &str) -> Result<()> {
        // Length check
        if name.is_empty() {
            return Err(MoteError::InvalidName("Name cannot be empty".to_string()));
        }
        if name.len() > 255 {
            return Err(MoteError::InvalidName(format!(
                "Name too long ({} chars, max 255)",
                name.len()
            )));
        }

        // Path traversal prevention
        if name.contains("..") || name.contains('/') || name.contains('\\') {
            return Err(MoteError::InvalidName(
                "Name cannot contain path separators or '..'".to_string(),
            ));
        }

        // Dot-only names prevention
        if name == "." {
            return Err(MoteError::InvalidName("Name cannot be '.'".to_string()));
        }

        // First character check (alphabetic or underscore, not digit)
        if let Some(first_char) = name.chars().next() {
            if !first_char.is_ascii_alphabetic() && first_char != '_' {
                return Err(MoteError::InvalidName(format!(
                    "Name must start with alphabetic character or underscore, got '{}'",
                    first_char
                )));
            }
        }

        // Character validation (alphanumeric, hyphen, underscore only)
        for (i, c) in name.chars().enumerate() {
            if !c.is_ascii_alphanumeric() && c != '-' && c != '_' {
                return Err(MoteError::InvalidName(format!(
                    "Invalid character '{}' at position {}",
                    c, i
                )));
            }
            // Control character check
            if c.is_control() {
                return Err(MoteError::InvalidName(
                    "Name cannot contain control characters".to_string(),
                ));
            }
        }

        // Null byte check
        if name.contains('\0') {
            return Err(MoteError::InvalidName(
                "Name cannot contain null bytes".to_string(),
            ));
        }

        // Windows reserved words check
        let upper_name = name.to_uppercase();
        let reserved_words = [
            "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7",
            "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8",
            "LPT9",
        ];
        if reserved_words.contains(&upper_name.as_str()) {
            return Err(MoteError::InvalidName(format!(
                "'{}' is a reserved word",
                name
            )));
        }

        Ok(())
    }
}
