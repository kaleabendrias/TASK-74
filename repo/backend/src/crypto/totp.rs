use totp_rs::{Algorithm, Secret, TOTP};

use crate::config::TotpConfig;
use crate::crypto::aes_gcm;

/// Verifies a TOTP code against an encrypted secret, decrypting it first with the master key.
pub fn verify(encrypted_secret: &[u8], code: &str, config: &TotpConfig, master_key: &str) -> bool {
    let secret_bytes = aes_gcm::decrypt(encrypted_secret, master_key);

    let totp = match TOTP::new(
        Algorithm::SHA1,
        config.digits as usize,
        1, // skew
        config.period_secs as u64,
        secret_bytes,
    ) {
        Ok(t) => t,
        Err(_) => return false,
    };

    totp.check_current(code).unwrap_or(false)
}

/// Generates a new random TOTP secret as raw bytes.
pub fn generate_secret() -> Vec<u8> {
    let secret = Secret::generate_secret();
    secret.to_bytes().expect("Failed to generate TOTP secret bytes")
}
