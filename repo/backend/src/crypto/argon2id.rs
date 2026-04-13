use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Algorithm, Argon2, Params, Version,
};

use crate::config::Argon2Config;

/// Hashes a password using Argon2id with the provided configuration parameters.
pub fn hash_with_config(password: &str, cfg: &Argon2Config) -> String {
    let salt = SaltString::generate(&mut OsRng);
    let params = Params::new(cfg.memory_kib, cfg.iterations, cfg.parallelism, Some(cfg.output_len))
        .expect("Invalid Argon2 parameters");
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    argon2
        .hash_password(password.as_bytes(), &salt)
        .expect("Failed to hash password")
        .to_string()
}

/// Hashes a password using Argon2id with default parameters.
pub fn hash(password: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .expect("Failed to hash password")
        .to_string()
}

/// Verifies a plaintext password against an Argon2id hash string.
pub fn verify(password: &str, hash: &str) -> bool {
    let parsed = match PasswordHash::new(hash) {
        Ok(h) => h,
        Err(_) => return false,
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok()
}
