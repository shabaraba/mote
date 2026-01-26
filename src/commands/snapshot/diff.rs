use std::collections::{HashMap, HashSet};
use std::fmt::Write;
use std::fs;
use std::path::Path;

use colored::*;
use similar::{ChangeTag, TextDiff};

use crate::commands::CommandContext;
use crate::error::{MoteError, Result};
use crate::ignore::IgnoreFilter;
use crate::storage::{FileEntry, ObjectStore, Snapshot, SnapshotStore, StorageLocation};

pub fn cmd_diff(
    ctx: &CommandContext,
    snapshot_id: Option<String>,
    snapshot_id2: Option<String>,
    name_only: bool,
    output: Option<String>,
    unified: usize,
) -> Result<()> {
    let location = match StorageLocation::find_existing(ctx.project_root, ctx.storage_dir) {
        Ok(loc) => loc,
        Err(MoteError::NotInitialized) if ctx.storage_dir.is_some() => {
            StorageLocation::init(ctx.project_root, ctx.config, ctx.storage_dir)?
        }
        Err(e) => return Err(e),
    };
    let snapshot_store = SnapshotStore::new(location.snapshots_dir());
    let object_store = ObjectStore::new(location.objects_dir());

    let snapshot_id = match snapshot_id {
        Some(id) => id,
        None => {
            let snapshots = snapshot_store.list()?;
            if snapshots.is_empty() {
                return Err(MoteError::ConfigRead("No snapshots found".to_string()));
            }
            snapshots.first().unwrap().id.clone()
        }
    };

    let snapshot1 = snapshot_store.find_by_id(&snapshot_id)?;
    let mut diff_output = String::new();

    if let Some(ref id2) = snapshot_id2 {
        let snapshot2 = snapshot_store.find_by_id(id2)?;
        diff_snapshots(
            &snapshot1,
            &snapshot2,
            &object_store,
            name_only,
            unified,
            &mut diff_output,
        )?;
    } else {
        diff_with_working_dir(
            ctx.project_root,
            &ctx.ignore_file_path,
            &snapshot1,
            &object_store,
            name_only,
            unified,
            &mut diff_output,
        )?;
    }

    if let Some(output_file) = output {
        fs::write(&output_file, &diff_output)?;
        println!("Diff written to {}", output_file.cyan());
    } else {
        print!("{}", diff_output);
    }

    Ok(())
}

fn files_to_map(files: &[FileEntry]) -> HashMap<&str, &FileEntry> {
    files.iter().map(|f| (f.path.as_str(), f)).collect()
}

fn diff_snapshots(
    snapshot1: &Snapshot,
    snapshot2: &Snapshot,
    object_store: &ObjectStore,
    name_only: bool,
    unified: usize,
    output: &mut String,
) -> Result<()> {
    writeln!(
        output,
        "Comparing {} -> {}",
        snapshot1.short_id(),
        snapshot2.short_id()
    )
    .unwrap();
    writeln!(output).unwrap();

    let files1 = files_to_map(&snapshot1.files);
    let files2 = files_to_map(&snapshot2.files);

    for (path, file2) in &files2 {
        if let Some(file1) = files1.get(path) {
            if file1.hash != file2.hash {
                if name_only {
                    writeln!(output, "M\t{}", path).unwrap();
                } else {
                    generate_unified_diff(
                        object_store,
                        path,
                        &file1.hash,
                        &file2.hash,
                        unified,
                        output,
                    )?;
                }
            }
        } else if name_only {
            writeln!(output, "A\t{}", path).unwrap();
        } else {
            generate_unified_diff(object_store, path, "", &file2.hash, unified, output)?;
        }
    }

    for path in files1.keys() {
        if !files2.contains_key(path) {
            if name_only {
                writeln!(output, "D\t{}", path).unwrap();
            } else {
                let file1 = files1.get(path).unwrap();
                generate_unified_diff(object_store, path, &file1.hash, "", unified, output)?;
            }
        }
    }
    Ok(())
}

fn diff_with_working_dir(
    project_root: &Path,
    ignore_file_path: &Path,
    snapshot: &Snapshot,
    object_store: &ObjectStore,
    name_only: bool,
    unified: usize,
    output: &mut String,
) -> Result<()> {
    writeln!(
        output,
        "Comparing {} -> working directory",
        snapshot.short_id()
    )
    .unwrap();
    writeln!(output).unwrap();

    let ignore_filter = IgnoreFilter::new(ignore_file_path);
    let snapshot_files = files_to_map(&snapshot.files);
    let mut current_files = HashSet::new();

    for entry in ignore_filter.walk_files(project_root) {
        let path = entry.path();
        let relative_path = path
            .strip_prefix(project_root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        current_files.insert(relative_path.clone());

        if let Some(snapshot_file) = snapshot_files.get(relative_path.as_str()) {
            let current_content = match fs::read(path) {
                Ok(content) => content,
                Err(e) => {
                    eprintln!(
                        "{}: Failed to read {}: {}",
                        "warning".yellow(),
                        relative_path,
                        e
                    );
                    continue;
                }
            };
            let current_hash = ObjectStore::compute_hash(&current_content);
            if current_hash != snapshot_file.hash {
                if name_only {
                    writeln!(output, "M\t{}", relative_path).unwrap();
                } else {
                    generate_unified_diff_with_content(
                        object_store,
                        &relative_path,
                        &snapshot_file.hash,
                        &current_content,
                        unified,
                        output,
                    )?;
                }
            }
        } else if name_only {
            writeln!(output, "A\t{}", relative_path).unwrap();
        } else {
            let current_content = match fs::read(path) {
                Ok(content) => content,
                Err(e) => {
                    eprintln!(
                        "{}: Failed to read {}: {}",
                        "warning".yellow(),
                        relative_path,
                        e
                    );
                    continue;
                }
            };
            generate_unified_diff_with_content(
                object_store,
                &relative_path,
                "",
                &current_content,
                unified,
                output,
            )?;
        }
    }

    for path in snapshot_files.keys() {
        if !current_files.contains(*path) {
            if name_only {
                writeln!(output, "D\t{}", path).unwrap();
            } else {
                let file = snapshot_files.get(path).unwrap();
                generate_unified_diff_with_content(
                    object_store,
                    path,
                    &file.hash,
                    &[],
                    unified,
                    output,
                )?;
            }
        }
    }
    Ok(())
}

fn generate_unified_diff(
    object_store: &ObjectStore,
    path: &str,
    hash1: &str,
    hash2: &str,
    context_lines: usize,
    output: &mut String,
) -> Result<()> {
    let content2 = if hash2.is_empty() {
        Vec::new()
    } else {
        match object_store.retrieve(hash2) {
            Ok(c) => c,
            Err(MoteError::ObjectNotFound(hash)) => {
                eprintln!(
                    "{}: Object not found for {}: {}",
                    "warning".yellow(),
                    path,
                    hash
                );
                return Ok(());
            }
            Err(e) => return Err(e),
        }
    };

    generate_unified_diff_with_content(object_store, path, hash1, &content2, context_lines, output)
}

fn generate_unified_diff_with_content(
    object_store: &ObjectStore,
    path: &str,
    hash1: &str,
    content2: &[u8],
    context_lines: usize,
    output: &mut String,
) -> Result<()> {
    let content1 = if hash1.is_empty() {
        Vec::new()
    } else {
        match object_store.retrieve(hash1) {
            Ok(c) => c,
            Err(MoteError::ObjectNotFound(hash)) => {
                eprintln!(
                    "{}: Object not found for {}: {}",
                    "warning".yellow(),
                    path,
                    hash
                );
                Vec::new()
            }
            Err(e) => return Err(e),
        }
    };

    let text1 = String::from_utf8_lossy(&content1);
    let text2 = String::from_utf8_lossy(content2);

    if text1.is_empty() && text2.is_empty() {
        return Ok(());
    }

    let diff = TextDiff::from_lines(&text1, &text2);

    writeln!(output, "diff --mote a/{} b/{}", path, path).unwrap();
    writeln!(output, "--- a/{}", path).unwrap();
    writeln!(output, "+++ b/{}", path).unwrap();

    for hunk in diff
        .unified_diff()
        .context_radius(context_lines)
        .iter_hunks()
    {
        write!(output, "{}", hunk.header()).unwrap();
        for change in hunk.iter_changes() {
            let sign = match change.tag() {
                ChangeTag::Delete => "-",
                ChangeTag::Insert => "+",
                ChangeTag::Equal => " ",
            };
            write!(output, "{}{}", sign, change.value()).unwrap();
        }
    }

    writeln!(output).unwrap();
    Ok(())
}
