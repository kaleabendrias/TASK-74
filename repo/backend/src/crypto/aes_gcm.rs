use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, AeadCore, Nonce,
};
use base64::{engine::general_purpose::STANDARD, Engine};

/// Encrypts plaintext using AES-256-GCM, returning nonce prepended to ciphertext.
pub fn encrypt(plaintext: &[u8], master_key_b64: &str) -> Vec<u8> {
    let key_bytes = STANDARD.decode(master_key_b64).expect("Invalid base64 master key");
    let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher.encrypt(&nonce, plaintext).expect("AES-GCM encryption failed");

    // Prepend nonce (12 bytes) to ciphertext
    let mut result = nonce.to_vec();
    result.extend_from_slice(&ciphertext);
    result
}

/// Decrypts AES-256-GCM data (nonce + ciphertext) using the base64-encoded master key.
pub fn decrypt(data: &[u8], master_key_b64: &str) -> Vec<u8> {
    let key_bytes = STANDARD.decode(master_key_b64).expect("Invalid base64 master key");
    let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);

    let (nonce_bytes, ciphertext) = data.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);
    cipher.decrypt(nonce, ciphertext).expect("AES-GCM decryption failed")
}
