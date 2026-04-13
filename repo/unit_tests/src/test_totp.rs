use base64::{engine::general_purpose::STANDARD, Engine};
use tourism_backend::config::TotpConfig;
use tourism_backend::crypto::{aes_gcm, totp};
use totp_rs::{Algorithm, Secret, TOTP};

fn test_key() -> String {
    STANDARD.encode([0x42u8; 32])
}

fn test_totp_config() -> TotpConfig {
    TotpConfig {
        issuer: "TestIssuer".to_string(),
        digits: 6,
        period_secs: 30,
    }
}

#[test]
fn verify_valid_totp_code() {
    let master_key = test_key();
    let config = test_totp_config();

    let secret = Secret::generate_secret();
    let secret_bytes = secret.to_bytes().unwrap();
    let encrypted = aes_gcm::encrypt(&secret_bytes, &master_key);

    let totp_instance = TOTP::new(
        Algorithm::SHA1, 6, 1, 30,
        secret_bytes.clone(),
    ).unwrap();
    let code = totp_instance.generate_current().unwrap();

    assert!(totp::verify(&encrypted, &code, &config, &master_key));
}

#[test]
fn verify_wrong_code_fails() {
    let master_key = test_key();
    let config = test_totp_config();

    let secret = Secret::generate_secret();
    let secret_bytes = secret.to_bytes().unwrap();
    let encrypted = aes_gcm::encrypt(&secret_bytes, &master_key);

    assert!(!totp::verify(&encrypted, "000000", &config, &master_key));
}

#[test]
fn verify_expired_code_fails() {
    let master_key = test_key();
    let config = test_totp_config();

    let secret = Secret::generate_secret();
    let secret_bytes = secret.to_bytes().unwrap();
    let encrypted = aes_gcm::encrypt(&secret_bytes, &master_key);

    let totp_instance = TOTP::new(
        Algorithm::SHA1, 6, 1, 30,
        secret_bytes,
    ).unwrap();
    let old_code = totp_instance.generate(0);

    assert!(!totp::verify(&encrypted, &old_code, &config, &master_key));
}

#[test]
fn generate_secret_produces_valid_bytes() {
    let s1 = totp::generate_secret();
    let s2 = totp::generate_secret();
    assert!(!s1.is_empty());
    assert!(!s2.is_empty());
    assert_ne!(s1, s2);
}

#[test]
fn verify_with_boundary_time_step() {
    let master_key = test_key();
    let config = test_totp_config();

    let secret = Secret::generate_secret();
    let secret_bytes = secret.to_bytes().unwrap();
    let encrypted = aes_gcm::encrypt(&secret_bytes, &master_key);

    let totp_instance = TOTP::new(
        Algorithm::SHA1, 6, 1, 30,
        secret_bytes,
    ).unwrap();

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let code = totp_instance.generate(now);
    assert!(totp::verify(&encrypted, &code, &config, &master_key));
}
