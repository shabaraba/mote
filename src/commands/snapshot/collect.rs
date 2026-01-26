use std::collections::HashMap;
use std::fs;
use std::path::Path;

use colored::*;

use crate::ignore::IgnoreFilter;
use crate::storage::{FileEntry, Index, IndexEntry, ObjectStore};

pub fn collect_files(
    project_root: &Path,
    ignore_file_path: &Path,
    object_store: &ObjectStore,
    index: &mut Index,
    quiet: bool,
) -> Vec<FileEntry> {
    let ignore_filter = IgnoreFilter::new(ignore_file_path);
    let mut files = Vec::new();

    for entry in ignore_filter.walk_files(project_root) {
        let path = entry.path();
        let relative_path = path
            .strip_prefix(project_root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        let metadata = match fs::symlink_metadata(path) {
            Ok(m) => m,
            Err(e) if !quiet => {
                eprintln!(
                    "{}: Failed to read metadata for {}: {}",
                    "warning".yellow(),
                    relative_path,
                    e
                );
                continue;
            }
            Err(_) => continue,
        };

        if metadata.file_type().is_symlink() {
            continue;
        }

        let mtime = match metadata.modified() {
            Ok(t) => t,
            Err(e) if !quiet => {
                eprintln!(
                    "{}: Failed to get mtime for {}: {}",
                    "warning".yellow(),
                    relative_path,
                    e
                );
                continue;
            }
            Err(_) => continue,
        };

        let size = metadata.len();

        if let Some(cached_entry) = index.is_unchanged(&relative_path, mtime, size) {
            files.push(FileEntry {
                path: relative_path,
                hash: cached_entry.hash.clone(),
                size: cached_entry.size,
                mode: None,
            });
            continue;
        }

        match object_store.store_file(path) {
            Ok((hash, file_size)) => {
                let entry = FileEntry {
                    path: relative_path.clone(),
                    hash: hash.clone(),
                    size: file_size,
                    mode: None,
                };

                index.insert(IndexEntry {
                    path: relative_path,
                    hash,
                    size: file_size,
                    mtime,
                });

                files.push(entry);
            }
            Err(e) if !quiet => {
                eprintln!(
                    "{}: Failed to store {}: {}",
                    "warning".yellow(),
                    relative_path,
                    e
                );
            }
            _ => {}
        }
    }
    files
}

pub fn have_same_file_hashes(files1: &[FileEntry], files2: &[FileEntry]) -> bool {
    if files1.len() != files2.len() {
        return false;
    }
    let map: HashMap<_, _> = files1.iter().map(|f| (&f.path, &f.hash)).collect();
    files2.iter().all(|f| map.get(&f.path) == Some(&&f.hash))
}
