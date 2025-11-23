use anyhow::{Context, Result};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use chacha20poly1305::{
    ChaCha20Poly1305, Key, Nonce,
    aead::{Aead, KeyInit},
};
use rand::RngCore;

const NONCE_SIZE: usize = 12;

pub struct Crypto {
    cipher: ChaCha20Poly1305,
}

impl Crypto {
    pub fn new(key: &[u8; 32]) -> Self {
        let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
        Self { cipher }
    }

    pub fn encrypt(&self, plaintext: &str) -> Result<String> {
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&ciphertext);

        Ok(format!("enc:{}", BASE64.encode(&result)))
    }

    pub fn decrypt(&self, encrypted: &str) -> Result<String> {
        let encrypted = encrypted
            .strip_prefix("enc:")
            .context("Invalid encrypted format")?;

        let data = BASE64
            .decode(encrypted)
            .context("Invalid base64 encoding")?;

        if data.len() < NONCE_SIZE {
            anyhow::bail!("Invalid encrypted data");
        }

        let nonce = Nonce::from_slice(&data[..NONCE_SIZE]);
        let ciphertext = &data[NONCE_SIZE..];

        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;

        String::from_utf8(plaintext).context("Invalid UTF-8 in decrypted data")
    }
}

pub fn generate_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut key);
    key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let key = generate_key();
        let crypto = Crypto::new(&key);

        let plaintext = "my-secret-password";
        let encrypted = crypto.encrypt(plaintext).unwrap();
        assert!(encrypted.starts_with("enc:"));

        let decrypted = crypto.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_different_nonces() {
        let key = generate_key();
        let crypto = Crypto::new(&key);

        let plaintext = "password";
        let enc1 = crypto.encrypt(plaintext).unwrap();
        let enc2 = crypto.encrypt(plaintext).unwrap();

        assert_ne!(enc1, enc2);
        assert_eq!(crypto.decrypt(&enc1).unwrap(), plaintext);
        assert_eq!(crypto.decrypt(&enc2).unwrap(), plaintext);
    }
}
