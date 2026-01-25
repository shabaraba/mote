//! Tests for configuration validation and resolution

#[cfg(test)]
mod validation_tests {
    use crate::config::{Config, ProjectConfig};
    use crate::error::MoteError;
    use std::path::PathBuf;

    fn create_test_project_config() -> ProjectConfig {
        ProjectConfig {
            path: PathBuf::from("/tmp/test"),
            config: Config::default(),
        }
    }

    #[test]
    fn test_validate_empty_name() {
        let config = create_test_project_config();
        let result = config.save(&PathBuf::from("/tmp/test"), "");
        assert!(result.is_err());
        if let Err(MoteError::InvalidName(msg)) = result {
            assert!(msg.contains("empty"));
        } else {
            panic!("Expected InvalidName error");
        }
    }

    #[test]
    fn test_validate_too_long_name() {
        let config = create_test_project_config();
        let name = "a".repeat(256);
        let result = config.save(&PathBuf::from("/tmp/test"), &name);
        assert!(result.is_err());
        if let Err(MoteError::InvalidName(msg)) = result {
            assert!(msg.contains("too long") || msg.contains("Too long"));
        } else {
            panic!("Expected InvalidName error");
        }
    }

    #[test]
    fn test_validate_max_length_name() {
        let config = create_test_project_config();
        let name = "a".repeat(255);
        let result = config.save(&PathBuf::from("/tmp/nonexistent_dir_for_test"), &name);
        // Should fail for filesystem reasons, not validation
        if let Err(MoteError::InvalidName(_)) = result {
            panic!("Should not fail validation for 255 char name");
        }
    }

    #[test]
    fn test_validate_path_traversal() {
        let config = create_test_project_config();

        let result = config.save(&PathBuf::from("/tmp/test"), "../etc/passwd");
        assert!(result.is_err());

        let result = config.save(&PathBuf::from("/tmp/test"), "../../secret");
        assert!(result.is_err());

        let result = config.save(&PathBuf::from("/tmp/test"), "foo/../bar");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_path_separators() {
        let config = create_test_project_config();

        let result = config.save(&PathBuf::from("/tmp/test"), "foo/bar");
        assert!(result.is_err());

        let result = config.save(&PathBuf::from("/tmp/test"), "foo\\bar");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_dot_names() {
        let config = create_test_project_config();

        let result = config.save(&PathBuf::from("/tmp/test"), ".");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_starts_with_number() {
        let config = create_test_project_config();
        let result = config.save(&PathBuf::from("/tmp/test"), "1project");
        assert!(result.is_err());
        if let Err(MoteError::InvalidName(msg)) = result {
            assert!(msg.contains("start"));
        } else {
            panic!("Expected InvalidName error for name starting with number");
        }
    }

    #[test]
    fn test_validate_starts_with_hyphen() {
        let config = create_test_project_config();
        let result = config.save(&PathBuf::from("/tmp/test"), "-project");
        assert!(result.is_err());
        if let Err(MoteError::InvalidName(msg)) = result {
            assert!(msg.contains("start"));
        } else {
            panic!("Expected InvalidName error for name starting with hyphen");
        }
    }

    #[test]
    fn test_validate_starts_with_underscore() {
        let config = create_test_project_config();
        let result = config.save(&PathBuf::from("/tmp/nonexistent"), "_project");
        // Should pass validation (fail on filesystem)
        if let Err(MoteError::InvalidName(_)) = result {
            panic!("Name starting with underscore should be valid");
        }
    }

    #[test]
    fn test_validate_invalid_chars() {
        let config = create_test_project_config();

        let result = config.save(&PathBuf::from("/tmp/test"), "pro ject");
        assert!(result.is_err());

        let result = config.save(&PathBuf::from("/tmp/test"), "pro@ject");
        assert!(result.is_err());

        let result = config.save(&PathBuf::from("/tmp/test"), "pro.ject");
        assert!(result.is_err());

        let result = config.save(&PathBuf::from("/tmp/test"), "pro$ject");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_valid_chars() {
        let config = create_test_project_config();

        // These should pass validation (will fail on filesystem)
        let names = vec!["pro-ject", "pro_ject", "project123", "Project", "_valid", "a"];

        for name in names {
            let result = config.save(&PathBuf::from("/tmp/nonexistent"), name);
            if let Err(MoteError::InvalidName(msg)) = result {
                panic!("Name '{}' should be valid, got error: {}", name, msg);
            }
        }
    }

    #[test]
    fn test_validate_reserved_words() {
        let config = create_test_project_config();

        let reserved = vec!["CON", "PRN", "AUX", "NUL", "COM1", "LPT1"];

        for word in reserved {
            let result = config.save(&PathBuf::from("/tmp/test"), word);
            assert!(result.is_err(), "Reserved word '{}' should be rejected", word);

            // Also test lowercase
            let result = config.save(&PathBuf::from("/tmp/test"), &word.to_lowercase());
            assert!(
                result.is_err(),
                "Reserved word '{}' (lowercase) should be rejected",
                word
            );
        }
    }

    #[test]
    fn test_validate_control_characters() {
        let config = create_test_project_config();

        let result = config.save(&PathBuf::from("/tmp/test"), "pro\nject");
        assert!(result.is_err());

        let result = config.save(&PathBuf::from("/tmp/test"), "pro\tject");
        assert!(result.is_err());

        let result = config.save(&PathBuf::from("/tmp/test"), "pro\x00ject");
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod context_validation_tests {
    use crate::config::{Config, ContextConfig};
    use crate::error::MoteError;
    use std::path::PathBuf;

    fn create_test_context_config() -> ContextConfig {
        ContextConfig {
            cwd: Some(PathBuf::from("/tmp/test")),
            storage_dir: None,
            config: Config::default(),
        }
    }

    #[test]
    fn test_context_validate_empty_name() {
        let config = create_test_context_config();
        let result = config.save(&PathBuf::from("/tmp/test"), "");
        assert!(result.is_err());
    }

    #[test]
    fn test_context_validate_path_traversal() {
        let config = create_test_context_config();
        let result = config.save(&PathBuf::from("/tmp/test"), "../escape");
        assert!(result.is_err());
    }

    #[test]
    fn test_context_validate_valid_names() {
        let config = create_test_context_config();

        let names = vec!["default", "feature-branch", "experiment_1", "_temp"];

        for name in names {
            let result = config.save(&PathBuf::from("/tmp/nonexistent"), name);
            if let Err(MoteError::InvalidName(msg)) = result {
                panic!("Context name '{}' should be valid, got error: {}", name, msg);
            }
        }
    }
}

#[cfg(test)]
mod config_merge_tests {
    use crate::config::{Config, ConfigResolver, ContextConfig, ProjectConfig, ResolveOptions};
    use std::path::PathBuf;

    #[test]
    fn test_resolve_global_only() {
        // When no project/context specified, should use global defaults
        let opts = ResolveOptions {
            config_dir: None,
            project: None,
            context: None,
            project_root: PathBuf::from("/tmp/test"),
        };

        // This will use default global config
        let result = ConfigResolver::load(&opts);
        assert!(result.is_ok());

        let resolver = result.unwrap();
        let _config = resolver.resolve();
        // Just verify it doesn't panic
    }

    #[test]
    fn test_resolve_options_structure() {
        // Test that ResolveOptions can be constructed correctly
        let opts = ResolveOptions {
            config_dir: Some(PathBuf::from("/custom/config")),
            project: Some("test-project".to_string()),
            context: Some("test-context".to_string()),
            project_root: PathBuf::from("/tmp/project"),
        };

        assert_eq!(opts.config_dir, Some(PathBuf::from("/custom/config")));
        assert_eq!(opts.project, Some("test-project".to_string()));
        assert_eq!(opts.context, Some("test-context".to_string()));
        assert_eq!(opts.project_root, PathBuf::from("/tmp/project"));
    }

    #[test]
    fn test_config_default_values() {
        // Test that Config has proper defaults
        let config = Config::default();

        // Should have default snapshot settings
        assert_eq!(config.snapshot.max_snapshots, 1000);
        assert_eq!(config.snapshot.max_age_days, 30);
        assert!(config.snapshot.auto_cleanup);

        // Should have default ignore file
        assert_eq!(config.ignore.ignore_file, ".moteignore");
    }

    #[test]
    fn test_project_config_structure() {
        // Test ProjectConfig construction
        let config = ProjectConfig {
            path: PathBuf::from("/path/to/project"),
            config: Config::default(),
        };

        assert_eq!(config.path, PathBuf::from("/path/to/project"));
    }

    #[test]
    fn test_context_config_structure() {
        // Test ContextConfig construction
        let config = ContextConfig {
            cwd: Some(PathBuf::from("/path/to/context")),
            storage_dir: Some("custom-storage".to_string()),
            config: Config::default(),
        };

        assert_eq!(config.cwd, Some(PathBuf::from("/path/to/context")));
        assert_eq!(config.storage_dir, Some("custom-storage".to_string()));
    }
}

#[cfg(test)]
mod integration_tests {
    use std::path::PathBuf;

    #[test]
    fn test_directory_structure_documentation() {
        // This test documents the expected directory structure
        // Actual file I/O is not performed, just structure validation

        let config_dir = PathBuf::from("/home/user/.config/mote");
        let project_name = "my-project";
        let context_name = "default";

        let project_dir = config_dir.join("projects").join(project_name);
        assert_eq!(
            project_dir,
            PathBuf::from("/home/user/.config/mote/projects/my-project")
        );

        let context_dir = project_dir.join("contexts").join(context_name);
        assert_eq!(
            context_dir,
            PathBuf::from("/home/user/.config/mote/projects/my-project/contexts/default")
        );

        let storage_dir = context_dir.join("storage");
        assert_eq!(
            storage_dir,
            PathBuf::from("/home/user/.config/mote/projects/my-project/contexts/default/storage")
        );
    }

    #[test]
    fn test_config_file_paths() {
        // Document expected config file paths
        let config_dir = PathBuf::from("/home/user/.config/mote");

        let global_config = config_dir.join("config.toml");
        assert_eq!(
            global_config,
            PathBuf::from("/home/user/.config/mote/config.toml")
        );

        let project_config = config_dir
            .join("projects")
            .join("my-project")
            .join("config.toml");
        assert_eq!(
            project_config,
            PathBuf::from("/home/user/.config/mote/projects/my-project/config.toml")
        );

        let context_config = config_dir
            .join("projects")
            .join("my-project")
            .join("contexts")
            .join("default")
            .join("config.toml");
        assert_eq!(
            context_config,
            PathBuf::from(
                "/home/user/.config/mote/projects/my-project/contexts/default/config.toml"
            )
        );
    }
}
