use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{MoteError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: String,
    pub hash: String,
    pub size: u64,
    #[serde(default)]
    pub mode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    #[serde(default)]
    pub message: Option<String>,
    pub files: Vec<FileEntry>,
    #[serde(default)]
    pub trigger: Option<String>,
}

impl Snapshot {
    pub fn new(files: Vec<FileEntry>, message: Option<String>, trigger: Option<String>) -> Self {
        let timestamp = Utc::now();
        let id = Self::generate_id(&timestamp, &files);

        Self {
            id,
            timestamp,
            message,
            files,
            trigger,
        }
    }

    fn generate_id(timestamp: &DateTime<Utc>, files: &[FileEntry]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(timestamp.to_rfc3339().as_bytes());
        for file in files {
            hasher.update(file.path.as_bytes());
            hasher.update(file.hash.as_bytes());
        }
        hex::encode(hasher.finalize())
    }

    pub fn short_id(&self) -> &str {
        &self.id[..7.min(self.id.len())]
    }

    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    pub fn find_file(&self, path: &str) -> Option<&FileEntry> {
        self.files.iter().find(|f| f.path == path)
    }
}

pub struct SnapshotStore {
    snapshots_dir: PathBuf,
}

impl SnapshotStore {
    pub fn new(snapshots_dir: PathBuf) -> Self {
        Self { snapshots_dir }
    }

    pub fn save(&self, snapshot: &Snapshot) -> Result<()> {
        let filename = format!(
            "{}_{}.json",
            snapshot.timestamp.format("%Y%m%d_%H%M%S"),
            &snapshot.id[..8.min(snapshot.id.len())]
        );
        let path = self.snapshots_dir.join(filename);

        let json = serde_json::to_string_pretty(snapshot)?;
        fs::write(&path, json)?;

        Ok(())
    }

    pub fn list(&self) -> Result<Vec<Snapshot>> {
        let mut snapshots = Vec::new();

        if !self.snapshots_dir.exists() {
            return Ok(snapshots);
        }

        for entry in fs::read_dir(&self.snapshots_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().map_or(false, |e| e == "json") {
                match self.load_snapshot(&path) {
                    Ok(snapshot) => snapshots.push(snapshot),
                    Err(e) => eprintln!("Warning: Failed to load snapshot {:?}: {}", path, e),
                }
            }
        }

        snapshots.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(snapshots)
    }

    fn load_snapshot(&self, path: &Path) -> Result<Snapshot> {
        let content = fs::read_to_string(path)?;
        let snapshot: Snapshot = serde_json::from_str(&content)?;
        Ok(snapshot)
    }

    pub fn find_by_id(&self, partial_id: &str) -> Result<Snapshot> {
        let snapshots = self.list()?;
        let matches: Vec<_> = snapshots
            .into_iter()
            .filter(|s| s.id.starts_with(partial_id))
            .collect();

        match matches.len() {
            0 => Err(MoteError::SnapshotNotFound(partial_id.to_string())),
            1 => Ok(matches.into_iter().next().unwrap()),
            _ => Err(MoteError::AmbiguousSnapshotId(partial_id.to_string())),
        }
    }

    pub fn latest(&self) -> Result<Option<Snapshot>> {
        let snapshots = self.list()?;
        Ok(snapshots.into_iter().next())
    }

    pub fn cleanup(&self, max_snapshots: u32, max_age_days: u32) -> Result<u32> {
        let mut snapshots = self.list()?;
        let now = Utc::now();
        let mut removed = 0;

        snapshots.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        for (i, snapshot) in snapshots.iter().enumerate() {
            let age_days = (now - snapshot.timestamp).num_days();
            let should_remove = i >= max_snapshots as usize || age_days > max_age_days as i64;

            if should_remove {
                if let Err(e) = self.remove(&snapshot.id) {
                    eprintln!("Warning: Failed to remove snapshot {}: {}", snapshot.short_id(), e);
                } else {
                    removed += 1;
                }
            }
        }

        Ok(removed)
    }

    fn remove(&self, id: &str) -> Result<()> {
        for entry in fs::read_dir(&self.snapshots_dir)? {
            let entry = entry?;
            let path = entry.path();

            if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
                if filename.contains(&id[..8.min(id.len())]) {
                    fs::remove_file(&path)?;
                    return Ok(());
                }
            }
        }
        Ok(())
    }
}
