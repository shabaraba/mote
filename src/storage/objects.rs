use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::error::{MoteError, Result};

pub struct ObjectStore {
    objects_dir: PathBuf,
    compression_level: i32,
}

impl ObjectStore {
    pub fn new(objects_dir: PathBuf, compression_level: i32) -> Self {
        Self {
            objects_dir,
            compression_level,
        }
    }

    pub fn store(&self, content: &[u8]) -> Result<String> {
        let hash = Self::compute_hash(content);
        let object_path = self.object_path(&hash);

        if object_path.exists() {
            return Ok(hash);
        }

        if let Some(parent) = object_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let compressed = zstd::encode_all(content, self.compression_level)?;
        fs::write(&object_path, compressed)?;

        Ok(hash)
    }

    pub fn retrieve(&self, hash: &str) -> Result<Vec<u8>> {
        let object_path = self.object_path(hash);

        if !object_path.exists() {
            return Err(MoteError::ObjectNotFound(hash.to_string()));
        }

        let compressed = fs::read(&object_path)?;
        let content = zstd::decode_all(compressed.as_slice())?;

        let actual_hash = Self::compute_hash(&content);
        if actual_hash != hash {
            return Err(MoteError::HashMismatch {
                expected: hash.to_string(),
                actual: actual_hash,
            });
        }

        Ok(content)
    }

    fn object_path(&self, hash: &str) -> PathBuf {
        let (prefix, rest) = hash.split_at(2);
        self.objects_dir.join(prefix).join(rest)
    }

    pub fn compute_hash(content: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content);
        hex::encode(hasher.finalize())
    }

    pub fn store_file(&self, path: &Path) -> Result<(String, u64)> {
        let mut file = File::open(path)?;
        let mut content = Vec::new();
        file.read_to_end(&mut content)?;

        let size = content.len() as u64;
        let hash = self.store(&content)?;

        Ok((hash, size))
    }

    pub fn restore_file(&self, hash: &str, dest: &Path) -> Result<()> {
        let content = self.retrieve(hash)?;

        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut file = File::create(dest)?;
        file.write_all(&content)?;

        Ok(())
    }
}
