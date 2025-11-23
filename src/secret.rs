use anyhow::{Context, Result};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use std::fs;
use std::path::PathBuf;

use crate::{config::ApplicationConfig, crypto};

pub struct SecretManager;

impl SecretManager {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }

    fn key_file_path() -> Result<PathBuf> {
        let mut path = ApplicationConfig::config_base_dir()
            .ok_or_else(|| anyhow::anyhow!("Cannot determine config directory"))?;
        path.push(".master.key");
        Ok(path)
    }

    pub fn get_or_create_master_key(&self) -> Result<[u8; 32]> {
        let key_path = Self::key_file_path()?;

        if key_path.exists() {
            let encoded = fs::read_to_string(&key_path).context("Failed to read master key")?;

            let decoded = BASE64
                .decode(encoded.trim())
                .context("Invalid master key encoding")?;

            if decoded.len() != 32 {
                anyhow::bail!("Invalid master key size");
            }

            let mut key = [0u8; 32];
            key.copy_from_slice(&decoded);
            Ok(key)
        } else {
            let key = crypto::generate_key();
            let encoded = BASE64.encode(key);

            if let Some(parent) = key_path.parent() {
                fs::create_dir_all(parent).context("Failed to create config directory")?;
            }

            fs::write(&key_path, encoded).context("Failed to store master key")?;

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&key_path)?.permissions();
                perms.set_mode(0o600);
                fs::set_permissions(&key_path, perms)?;
            }

            Ok(key)
        }
    }

    pub fn reset_master_key(&self) -> Result<()> {
        let key_path = Self::key_file_path()?;
        if key_path.exists() {
            fs::remove_file(&key_path).context("Failed to delete master key")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_master_key_persistence() {
        let manager = SecretManager::new().unwrap();

        let key1 = manager.get_or_create_master_key().unwrap();
        let key2 = manager.get_or_create_master_key().unwrap();

        assert_eq!(key1, key2);

        manager.reset_master_key().unwrap();
    }
}
