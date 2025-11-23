use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use keyring::Entry;

use crate::crypto;

const SERVICE_NAME: &str = "rds-cli";
const KEY_NAME: &str = "master-key";

pub struct SecretManager {
    entry: Entry,
}

impl SecretManager {
    pub fn new() -> Result<Self> {
        let entry = Entry::new(SERVICE_NAME, KEY_NAME)
            .context("Failed to access keyring")?;
        Ok(Self { entry })
    }

    pub fn get_or_create_master_key(&self) -> Result<[u8; 32]> {
        match self.entry.get_password() {
            Ok(encoded) => {
                let decoded = BASE64
                    .decode(encoded)
                    .context("Invalid master key encoding")?;

                if decoded.len() != 32 {
                    anyhow::bail!("Invalid master key size");
                }

                let mut key = [0u8; 32];
                key.copy_from_slice(&decoded);
                Ok(key)
            }
            Err(_) => {
                let key = crypto::generate_key();
                let encoded = BASE64.encode(&key);
                self.entry
                    .set_password(&encoded)
                    .context("Failed to store master key")?;
                Ok(key)
            }
        }
    }

    pub fn reset_master_key(&self) -> Result<()> {
        self.entry
            .delete_credential()
            .context("Failed to delete master key")?;
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
