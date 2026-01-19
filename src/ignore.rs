use ignore::gitignore::{Gitignore, GitignoreBuilder};
use std::path::Path;
use walkdir::WalkDir;

use crate::error::Result;

pub struct IgnoreFilter {
    gitignore: Option<Gitignore>,
}

impl IgnoreFilter {
    pub fn new(project_root: &Path, ignore_file: &str) -> Self {
        let ignore_path = project_root.join(ignore_file);

        let gitignore = if ignore_path.exists() {
            let mut builder = GitignoreBuilder::new(project_root);
            let _ = builder.add(&ignore_path);
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

pub fn create_default_moteignore(project_root: &Path) -> Result<()> {
    let ignore_path = project_root.join(".moteignore");

    if ignore_path.exists() {
        return Ok(());
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

    std::fs::write(&ignore_path, default_content)?;
    Ok(())
}
