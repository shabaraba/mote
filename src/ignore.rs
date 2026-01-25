use ignore::gitignore::{Gitignore, GitignoreBuilder};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::error::Result;

pub struct IgnoreFilter {
    gitignore: Option<Gitignore>,
}

impl IgnoreFilter {
    /// Creates a new IgnoreFilter for the given ignore file path.
    ///
    /// # Arguments
    /// * `ignore_file_path` - Full path to the ignore file
    pub fn new(ignore_file_path: &Path) -> Self {
        let gitignore = if ignore_file_path.exists() {
            // Use parent directory as project root for gitignore rules
            let project_root = ignore_file_path
                .parent()
                .unwrap_or_else(|| Path::new("."));

            let mut builder = GitignoreBuilder::new(project_root);
            let _ = builder.add(ignore_file_path);
            builder.build().ok()
        } else {
            None
        };

        Self { gitignore }
    }

    pub fn is_ignored(&self, path: &Path, is_dir: bool) -> bool {
        if let Some(ref gi) = self.gitignore {
            gi.matched(path, is_dir).is_ignore()
        } else {
            false
        }
    }

    pub fn walk_files(&self, project_root: &Path) -> Vec<walkdir::DirEntry> {
        let mote_dir = project_root.join(".mote");
        let git_dir = project_root.join(".git");
        let jj_dir = project_root.join(".jj");

        WalkDir::new(project_root)
            .into_iter()
            .filter_entry(|entry| {
                let path = entry.path();

                if path.starts_with(&mote_dir)
                    || path.starts_with(&git_dir)
                    || path.starts_with(&jj_dir)
                {
                    return false;
                }

                let relative_path = path.strip_prefix(project_root).unwrap_or(path);
                !self.is_ignored(relative_path, entry.file_type().is_dir())
            })
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .collect()
    }
}

/// Creates an ignore file at the specified path with default content.
/// Returns the path of the created file (or existing file if already present).
///
/// # Arguments
/// * `ignore_path` - The full path where the ignore file should be created
///
/// # Behavior
/// - Does not overwrite existing files (idempotent)
/// - Automatically creates parent directories if they don't exist
/// - Returns Ok even if file already exists
pub fn create_ignore_file(ignore_path: &Path) -> Result<PathBuf> {
    // Don't overwrite existing files
    if ignore_path.exists() {
        return Ok(ignore_path.to_path_buf());
    }

    // Create parent directories if needed
    if let Some(parent) = ignore_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let default_content = r#"# Mote ignore file
# Uses gitignore syntax

# Dependencies
node_modules/
vendor/
.venv/
venv/
__pycache__/

# Build outputs
target/
dist/
build/
*.o
*.a
*.so
*.dylib

# IDE and editor
.idea/
.vscode/
*.swp
*.swo
*~

# OS files
.DS_Store
Thumbs.db

# Logs
*.log
logs/

# Temporary files
*.tmp
*.temp
.cache/
"#;

    std::fs::write(ignore_path, default_content)?;
    Ok(ignore_path.to_path_buf())
}

