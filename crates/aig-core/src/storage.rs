use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

pub struct BlobStore {
    base_path: PathBuf,
}

impl BlobStore {
    pub fn new(path: &Path) -> Result<Self> {
        let base_path = path.join("objects");
        fs::create_dir_all(&base_path)?;
        Ok(Self { base_path })
    }

    pub fn store_blob(&self, data: &[u8]) -> Result<String> {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let hash = hex::encode(hasher.finalize());

        let dir = self.base_path.join(&hash[..2]);
        fs::create_dir_all(&dir)?;
        let file_path = dir.join(&hash[2..]);
        fs::write(&file_path, data)?;

        Ok(hash)
    }

    pub fn get_blob(&self, hash: &str) -> Result<Vec<u8>> {
        let file_path = self.base_path.join(&hash[..2]).join(&hash[2..]);
        fs::read(&file_path).with_context(|| format!("blob not found: {hash}"))
    }
}
