use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::SystemTime;

use crate::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexEntry {
    pub path: String,
    pub hash: String,
    pub size: u64,
    #[serde(with = "systemtime_serde")]
    pub mtime: SystemTime,
}

mod systemtime_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    #[derive(Serialize, Deserialize)]
    struct SystemTimeData {
        secs: u64,
        nanos: u32,
    }

    pub fn serialize<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let duration = time.duration_since(UNIX_EPOCH).map_err(serde::ser::Error::custom)?;
        let data = SystemTimeData {
            secs: duration.as_secs(),
            nanos: duration.subsec_nanos(),
        };
        data.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let data = SystemTimeData::deserialize(deserializer)?;
        Ok(UNIX_EPOCH + Duration::new(data.secs, data.nanos))
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Index {
    entries: HashMap<String, IndexEntry>,
}

impl Index {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn load(index_path: &Path) -> Result<Self> {
        if !index_path.exists() {
            return Ok(Self::new());
        }

        let content = fs::read(index_path)?;
        let index: Index = bincode::deserialize(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(index)
    }

    pub fn save(&self, index_path: &Path) -> Result<()> {
        if let Some(parent) = index_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let encoded = bincode::serialize(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        fs::write(index_path, encoded)?;
        Ok(())
    }

    pub fn insert(&mut self, entry: IndexEntry) {
        self.entries.insert(entry.path.clone(), entry);
    }

    pub fn is_unchanged(&self, path: &str, mtime: SystemTime, size: u64) -> Option<&IndexEntry> {
        self.entries.get(path).and_then(|entry| {
            if entry.mtime == mtime && entry.size == size {
                Some(entry)
            } else {
                None
            }
        })
    }
}
