use std::path::{Path, PathBuf};

/// Resolves a path relative to a base directory.
/// If the path is absolute, returns it as-is.
/// If the path is relative, joins it with the base directory.
pub fn resolve_path(base: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    }
}

/// Resolves the ignore file path with default fallback.
/// If cli_path is provided, resolves it relative to project_root.
/// Otherwise, returns project_root joined with default_name.
pub fn resolve_ignore_file_path(
    project_root: &Path,
    cli_path: Option<&Path>,
    default_name: &str,
) -> PathBuf {
    match cli_path {
        Some(path) => resolve_path(project_root, path),
        None => project_root.join(default_name),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_resolve_path_absolute() {
        let base = Path::new("/base");
        let path = Path::new("/absolute/path");
        assert_eq!(resolve_path(base, path), PathBuf::from("/absolute/path"));
    }

    #[test]
    fn test_resolve_path_relative() {
        let base = Path::new("/base");
        let path = Path::new("relative/path");
        assert_eq!(
            resolve_path(base, path),
            PathBuf::from("/base/relative/path")
        );
    }

    #[test]
    fn test_resolve_ignore_file_path_with_cli() {
        let project_root = Path::new("/project");
        let cli_path = Some(Path::new("custom/my.ignore"));
        let result = resolve_ignore_file_path(project_root, cli_path, ".moteignore");
        assert_eq!(result, PathBuf::from("/project/custom/my.ignore"));
    }

    #[test]
    fn test_resolve_ignore_file_path_without_cli() {
        let project_root = Path::new("/project");
        let result = resolve_ignore_file_path(project_root, None, ".moteignore");
        assert_eq!(result, PathBuf::from("/project/.moteignore"));
    }

    #[test]
    fn test_resolve_ignore_file_path_absolute() {
        let project_root = Path::new("/project");
        let cli_path = Some(Path::new("/tmp/my.ignore"));
        let result = resolve_ignore_file_path(project_root, cli_path, ".moteignore");
        assert_eq!(result, PathBuf::from("/tmp/my.ignore"));
    }
}
