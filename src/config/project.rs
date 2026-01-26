use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::error::{MoteError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub path: PathBuf,
    /// Map of context name -> context directory path
    /// Used to track contexts with custom context_dir
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contexts: Option<HashMap<String, PathBuf>>,
    #[serde(flatten)]
    pub config: Config,
}

impl ProjectConfig {
    /// Load project configuration from config directory
    pub fn load(config_dir: &Path, project_name: &str) -> Result<Self> {
        Self::validate_name(project_name)?;

        let project_dir = config_dir.join("projects").join(project_name);
        let config_path = project_dir.join("config.toml");

        if !config_path.exists() {
            return Err(MoteError::ProjectNotFound(project_name.to_string()));
        }

        let content =
            fs::read_to_string(&config_path).map_err(|e| MoteError::ConfigRead(e.to_string()))?;

        let config: ProjectConfig = toml::from_str(&content)?;
        Ok(config)
    }

    /// Save project configuration
    /// If allow_overwrite is true, overwrites existing config (used for updating contexts map)
    pub fn save(&self, config_dir: &Path, project_name: &str) -> Result<()> {
        Self::validate_name(project_name)?;

        let project_dir = config_dir.join("projects").join(project_name);

        if !project_dir.exists() {
            fs::create_dir_all(&project_dir)?;
        }

        let config_path = project_dir.join("config.toml");

        let content = toml::to_string_pretty(self)
            .map_err(|e| MoteError::ConfigParse(e.to_string()))?;

        fs::write(&config_path, content)?;
        Ok(())
    }

    /// Register a context with custom directory
    pub fn register_context(&mut self, context_name: String, context_dir: PathBuf) {
        if self.contexts.is_none() {
            self.contexts = Some(HashMap::new());
        }
        if let Some(ref mut contexts) = self.contexts {
            contexts.insert(context_name, context_dir);
        }
    }

    /// Unregister a context
    pub fn unregister_context(&mut self, context_name: &str) {
        if let Some(ref mut contexts) = self.contexts {
            contexts.remove(context_name);
        }
    }

    /// Get context directory (returns custom dir if registered, otherwise default)
    pub fn get_context_dir(&self, project_dir: &Path, context_name: &str) -> PathBuf {
        if let Some(ref contexts) = self.contexts {
            if let Some(custom_dir) = contexts.get(context_name) {
                return custom_dir.clone();
            }
        }
        // Default location
        project_dir.join("contexts").join(context_name)
    }

    /// List all project names in config directory
    pub fn list(config_dir: &Path) -> Result<Vec<String>> {
        let projects_dir = config_dir.join("projects");

        if !projects_dir.exists() {
            return Ok(Vec::new());
        }

        let mut names = Vec::new();

        for entry in fs::read_dir(projects_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    names.push(name.to_string());
                }
            }
        }

        Ok(names)
    }

    /// Find project by path
    pub fn find_by_path(config_dir: &Path, project_path: &Path) -> Result<Option<String>> {
        let canonical_path = project_path
            .canonicalize()
            .unwrap_or_else(|_| project_path.to_path_buf());

        for project_name in Self::list(config_dir)? {
            if let Ok(project) = Self::load(config_dir, &project_name) {
                let project_canonical = project
                    .path
                    .canonicalize()
                    .unwrap_or_else(|_| project.path.clone());

                if project_canonical == canonical_path {
                    return Ok(Some(project_name));
                }
            }
        }

        Ok(None)
    }

    /// Validate project/context name with comprehensive security checks
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
