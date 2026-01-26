use std::path::Path;

use colored::*;

use crate::config::{Config, ConfigResolver, ContextConfig, ProjectConfig};
use crate::error::Result;
use crate::ignore::create_ignore_file;

pub fn cmd_migrate(
    project_root: &Path,
    config_resolver: &ConfigResolver,
    dry_run: bool,
) -> Result<()> {
    let old_mote_dir = project_root.join(".mote");

    if !old_mote_dir.exists() {
        println!(
            "{} No .mote directory found to migrate",
            "!".yellow().bold()
        );
        return Ok(());
    }

    let mut project_name = project_root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("migrated-project")
        .to_string();

    project_name = sanitize_project_name(&project_name);

    println!("Migrating .mote/ to new structure...");
    println!("  Project name: {}", project_name.cyan());
    println!("  Source: {}", old_mote_dir.display());

    let config_dir = config_resolver.config_dir();
    let new_project_dir = config_dir.join("projects").join(&project_name);
    let new_context_dir = new_project_dir.join("contexts").join("default");
    let new_storage_dir = new_context_dir.join("storage");

    println!("  Destination: {}", new_storage_dir.display());

    if dry_run {
        println!("\n{} Dry run - no changes made", "i".cyan().bold());
        return Ok(());
    }

    std::fs::create_dir_all(&new_storage_dir)?;

    let project_config = ProjectConfig {
        path: project_root
            .canonicalize()
            .unwrap_or_else(|_| project_root.to_path_buf()),
        contexts: None,
        config: Config::default(),
    };
    project_config.save(config_dir, &project_name)?;

    let context_config = ContextConfig {
        cwd: Some(project_root.to_path_buf()),
        context_dir: None,
        config: Config::default(),
    };
    context_config.save(&new_project_dir, "default")?;

    for entry in std::fs::read_dir(&old_mote_dir)? {
        let entry = entry?;
        let dest = new_storage_dir.join(entry.file_name());
        if entry.path().is_dir() {
            copy_dir_all(&entry.path(), &dest)?;
        } else {
            std::fs::copy(&entry.path(), &dest)?;
        }
    }

    let old_ignore = project_root.join(".moteignore");
    let new_ignore = new_context_dir.join("ignore");

    if old_ignore.exists() {
        std::fs::copy(&old_ignore, &new_ignore)?;
        println!("  Copied .moteignore to context");
    } else {
        create_ignore_file(&new_ignore)?;
    }

    println!("\n{} Migration complete!", "✓".green().bold());
    println!("  You can now remove the old .mote/ directory");
    println!("  Use: -p {} -c default for future commands", project_name);

    Ok(())
}

fn sanitize_project_name(name: &str) -> String {
    let mut sanitized = String::new();

    for c in name.chars() {
        if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
            sanitized.push(c);
        } else {
            sanitized.push('_');
        }
    }

    if sanitized.is_empty() || sanitized.starts_with(char::is_numeric) {
        sanitized.insert(0, 'm');
    }

    if sanitized.len() > 255 {
        sanitized.truncate(255);
    }

    sanitized
}

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    let src_canonical = src.canonicalize()?;
    std::fs::create_dir_all(dst)?;

    if let Ok(dst_canonical) = dst.canonicalize() {
        if dst_canonical.starts_with(&src_canonical) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Destination cannot be a subdirectory of source",
            ));
        }
    }

    copy_dir_all_impl(&src_canonical, dst)
}

fn copy_dir_all_impl(src: &Path, dst: &Path) -> std::io::Result<()> {
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        let file_type = entry.file_type()?;

        if file_type.is_symlink() {
            eprintln!("Warning: Skipping symbolic link: {:?}", src_path);
            continue;
        }

        if file_type.is_dir() {
            std::fs::create_dir_all(&dst_path)?;
            copy_dir_all_impl(&src_path, &dst_path)?;
        } else if file_type.is_file() {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}
