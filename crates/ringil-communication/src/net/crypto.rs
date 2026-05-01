use crate::protocol::pb::RingilFrame;
use anyhow::{Context, Result};
use chacha20poly1305::{
    ChaCha20Poly1305, Key, Nonce,
    aead::{Aead, KeyInit},
};
use prost::Message;
use rand::Rng;

/// Zero-trust payload encryptor using ChaCha20-Poly1305 (AEAD).
/// Adds 28 bytes of overhead (12 byte nonce + 16 byte MAC).
pub struct PayloadEncryptor {
    cipher: ChaCha20Poly1305,
}

impl PayloadEncryptor {
    /// Initialize from a 32-byte key (extracted from TPM by NixOS).
    pub fn new(key_bytes: &[u8; 32]) -> Self {
        let key = Key::from_slice(key_bytes);
        Self {
            cipher: ChaCha20Poly1305::new(key),
        }
    }

    /// Serialize and encrypt a Protobuf frame.
    pub fn encrypt(&self, frame: &RingilFrame) -> Result<Vec<u8>> {
        let mut buf = Vec::with_capacity(frame.encoded_len());
        frame.encode(&mut buf)?;

        let mut nonce_bytes = [0u8; 12];
        rand::rng().fill_bytes(&mut nonce_bytes);

        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = self
            .cipher
            .encrypt(nonce, buf.as_ref())
            .map_err(|_| anyhow::anyhow!("Encryption failed"))?;

        // Prepend nonce to ciphertext for decryption.
        let mut final_payload = Vec::with_capacity(12 + ciphertext.len());
        final_payload.extend_from_slice(&nonce_bytes);
        final_payload.extend(ciphertext);

        Ok(final_payload)
    }

    /// Decrypt and deserialize a Protobuf frame.
    pub fn decrypt(&self, payload: &[u8]) -> Result<RingilFrame> {
        if payload.len() < 28 {
            anyhow::bail!("Payload too short, missing nonce or MAC");
        }

        let nonce = Nonce::from_slice(&payload[..12]);
        let ciphertext = &payload[12..];

        let plaintext =
            self.cipher.decrypt(nonce, ciphertext).map_err(|_| {
                anyhow::anyhow!(
                    "Decryption/Authentication failed (Compromised?)"
                )
            })?;

        let frame = RingilFrame::decode(plaintext.as_slice())
            .context("Failed to decode Protobuf frame")?;

        Ok(frame)
    }
}
