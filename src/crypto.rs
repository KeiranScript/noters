use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::Rng;
use sha2::{Digest, Sha256};
use crate::error::{NoterError, Result};

pub struct Crypto {
    cipher: Aes256Gcm,
}

impl Crypto {
    pub fn new(key: &str) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        let key = hasher.finalize();
        let cipher = Aes256Gcm::new_from_slice(&key).expect("Invalid key length");
        Self { cipher }
    }

    pub fn encrypt(&self, data: &[u8]) -> Result<String> {
        let mut rng = rand::thread_rng();
        let mut nonce_bytes = [0u8; 12];
        rng.fill(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = self.cipher
            .encrypt(nonce, data)
            .map_err(|e| NoterError::Encryption(e.to_string()))?;
        
        let mut combined = nonce_bytes.to_vec();
        combined.extend(ciphertext);
        Ok(BASE64.encode(combined))
    }

    pub fn decrypt(&self, data: &str) -> Result<Vec<u8>> {
        let decoded = BASE64.decode(data)
            .map_err(|e| NoterError::Encryption(e.to_string()))?;
            
        if decoded.len() < 12 {
            return Err(NoterError::Encryption("Invalid encrypted data".to_string()));
        }

        let (nonce_bytes, ciphertext) = decoded.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| NoterError::Encryption(e.to_string()))
    }
}
