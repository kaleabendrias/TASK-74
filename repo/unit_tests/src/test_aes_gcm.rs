use base64::{engine::general_purpose::STANDARD, Engine};
use tourism_backend::crypto::aes_gcm;

fn test_key() -> String {
    // Generate a valid 32-byte key encoded as base64
    let key = [0x42u8; 32];
    STANDARD.encode(key)
}

#[test]
fn encrypt_decrypt_roundtrip() {
    let key = test_key();
    let plaintext = b"Hello, World!";
    let ciphertext = aes_gcm::encrypt(plaintext, &key);
    let decrypted = aes_gcm::decrypt(&ciphertext, &key);
    assert_eq!(decrypted, plaintext);
}

#[test]
fn encrypt_produces_different_ciphertext_each_time() {
    let key = test_key();
    let plaintext = b"same data";
    let c1 = aes_gcm::encrypt(plaintext, &key);
    let c2 = aes_gcm::encrypt(plaintext, &key);
    // Due to random nonce, ciphertexts differ
    assert_ne!(c1, c2);
    // But both decrypt to the same plaintext
    assert_eq!(aes_gcm::decrypt(&c1, &key), plaintext);
    assert_eq!(aes_gcm::decrypt(&c2, &key), plaintext);
}

#[test]
fn ciphertext_longer_than_plaintext() {
    let key = test_key();
    let plaintext = b"short";
    let ciphertext = aes_gcm::encrypt(plaintext, &key);
    // ciphertext = 12 byte nonce + plaintext + 16 byte auth tag
    assert_eq!(ciphertext.len(), 12 + plaintext.len() + 16);
}

#[test]
fn empty_plaintext_roundtrip() {
    let key = test_key();
    let ciphertext = aes_gcm::encrypt(b"", &key);
    let decrypted = aes_gcm::decrypt(&ciphertext, &key);
    assert_eq!(decrypted, b"");
}

#[test]
fn large_plaintext_roundtrip() {
    let key = test_key();
    let plaintext = vec![0xABu8; 10_000];
    let ciphertext = aes_gcm::encrypt(&plaintext, &key);
    let decrypted = aes_gcm::decrypt(&ciphertext, &key);
    assert_eq!(decrypted, plaintext);
}

#[test]
#[should_panic(expected = "AES-GCM decryption failed")]
fn corrupted_ciphertext_panics() {
    let key = test_key();
    let plaintext = b"secret data";
    let mut ciphertext = aes_gcm::encrypt(plaintext, &key);
    // Corrupt a byte in the ciphertext portion (after the 12-byte nonce)
    if ciphertext.len() > 14 {
        ciphertext[14] ^= 0xFF;
    }
    aes_gcm::decrypt(&ciphertext, &key);
}

#[test]
#[should_panic(expected = "AES-GCM decryption failed")]
fn truncated_ciphertext_panics() {
    let key = test_key();
    let plaintext = b"secret data";
    let ciphertext = aes_gcm::encrypt(plaintext, &key);
    // Truncate — remove last few bytes (breaks auth tag)
    let truncated = &ciphertext[..ciphertext.len() - 4];
    aes_gcm::decrypt(truncated, &key);
}

#[test]
#[should_panic]
fn wrong_key_panics_on_decrypt() {
    let key1 = test_key();
    let key2 = {
        let k = [0x43u8; 32];
        STANDARD.encode(k)
    };
    let ciphertext = aes_gcm::encrypt(b"data", &key1);
    aes_gcm::decrypt(&ciphertext, &key2);
}

#[test]
fn binary_data_roundtrip() {
    let key = test_key();
    let plaintext: Vec<u8> = (0..=255).collect();
    let ciphertext = aes_gcm::encrypt(&plaintext, &key);
    let decrypted = aes_gcm::decrypt(&ciphertext, &key);
    assert_eq!(decrypted, plaintext);
}
