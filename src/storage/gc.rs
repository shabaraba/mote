use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::error::Result;
use crate::storage::Snapshot;

pub struct ObjectReferences {
    refs: HashSet<String>,
}

impl ObjectReferences {
    pub fn new() -> Self {
        Self {
            refs: HashSet::new(),
        }
    }

    pub fn mark_from_snapshot(&mut self, snapshot: &Snapshot) {
        for file in &snapshot.files {
            self.refs.insert(file.hash.clone());
        }
    }

    pub fn is_referenced(&self, hash: &str) -> bool {
        self.refs.contains(hash)
    }

    pub fn referenced_count(&self) -> usize {
        self.refs.len()
    }
}

pub struct GcStats {
    pub deleted_objects: usize,
    pub deleted_bytes: u64,
}

pub fn list_all_objects(objects_dir: &Path) -> Result<Vec<String>> {
    let mut objects = Vec::new();

    if !objects_dir.exists() {
        return Ok(objects);
    }

    for prefix_entry in fs::read_dir(objects_dir)? {
        let prefix_entry = prefix_entry?;
        let prefix_path = prefix_entry.path();

        if !prefix_path.is_dir() {
            continue;
        }

        let prefix = prefix_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        for object_entry in fs::read_dir(&prefix_path)? {
            let object_entry = object_entry?;
            let object_name = object_entry
                .file_name()
                .to_string_lossy()
                .to_string();

            let hash = format!("{}{}", prefix, object_name);
            objects.push(hash);
        }
    }

    Ok(objects)
}

pub fn delete_objects(
    objects_dir: &Path,
    hashes_to_delete: &[String],
    verbose: bool,
) -> Result<GcStats> {
    let mut deleted_objects = 0;
    let mut deleted_bytes = 0;

    for hash in hashes_to_delete {
        if hash.len() < 2 {
            eprintln!("Warning: Skipping invalid hash: {}", hash);
            continue;
        }

        let (prefix, rest) = hash.split_at(2);
        let object_path = objects_dir.join(prefix).join(rest);

        if !object_path.exists() {
            continue;
        }

        let size = fs::metadata(&object_path)?.len();

        if verbose {
            println!("  Deleting object: {}", hash);
        }

        fs::remove_file(&object_path)?;
        deleted_objects += 1;
        deleted_bytes += size;

        let prefix_dir = objects_dir.join(prefix);
        if let Ok(mut entries) = fs::read_dir(&prefix_dir) {
            if entries.next().is_none() {
                let _ = fs::remove_dir(&prefix_dir);
            }
        }
    }

    Ok(GcStats {
        deleted_objects,
        deleted_bytes,
    })
}
