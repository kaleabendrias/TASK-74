use totp_rs::{Algorithm, TOTP, Secret};

use crate::config::TotpConfig;
use crate::crypto::aes_gcm;

pub fn verify(encrypted_secret: &[u8], code: &str, config: &TotpConfig, master_key: &str) -> bool {
    let secret_bytes = aes_gcm::decrypt(encrypted_secret, master_key);
    let secret = match Secret::Raw(secret_bytes).to_bytes() {
        Ok(b) => b,
        Err(_) => return false,
    };

    let totp = match TOTP::new(
        Algorithm::SHA1,
        config.digits as usize,
        1, // skew
        config.period_secs,
        secret,
        Some(config.issuer.clone()),
        String::new(),
    ) {
        Ok(t) => t,
        Err(_) => return false,
    };

    totp.check_current(code).unwrap_or(false)
}

pub fn generate_secret() -> Vec<u8> {
    let secret = Secret::generate_secret();
    secret.to_bytes().expect("Failed to generate TOTP secret bytes")
}
