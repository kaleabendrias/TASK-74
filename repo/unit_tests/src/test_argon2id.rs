use tourism_backend::config::Argon2Config;
use tourism_backend::crypto::argon2id;

#[test]
fn hash_and_verify_succeeds() {
    let hash = argon2id::hash("correct-horse-battery-staple");
    assert!(argon2id::verify("correct-horse-battery-staple", &hash));
}

#[test]
fn verify_wrong_password_fails() {
    let hash = argon2id::hash("mypassword");
    assert!(!argon2id::verify("wrongpassword", &hash));
}

#[test]
fn hash_with_config_and_verify() {
    let cfg = Argon2Config {
        memory_kib: 4096,
        iterations: 1,
        parallelism: 1,
        output_len: 32,
    };
    let hash = argon2id::hash_with_config("test123", &cfg);
    assert!(argon2id::verify("test123", &hash));
    assert!(!argon2id::verify("test124", &hash));
}

#[test]
fn verify_invalid_hash_string_returns_false() {
    assert!(!argon2id::verify("anything", "not-a-valid-hash"));
}

#[test]
fn verify_empty_hash_returns_false() {
    assert!(!argon2id::verify("anything", ""));
}

#[test]
fn different_passwords_produce_different_hashes() {
    let h1 = argon2id::hash("password1");
    let h2 = argon2id::hash("password2");
    assert_ne!(h1, h2);
}

#[test]
fn same_password_produces_different_hashes_due_to_salt() {
    let h1 = argon2id::hash("same-password");
    let h2 = argon2id::hash("same-password");
    assert_ne!(h1, h2);
    // But both verify
    assert!(argon2id::verify("same-password", &h1));
    assert!(argon2id::verify("same-password", &h2));
}

#[test]
fn hash_contains_argon2id_identifier() {
    let hash = argon2id::hash("test");
    assert!(hash.contains("$argon2id$"), "Hash should contain argon2id identifier: {}", hash);
}

#[test]
fn unicode_password_roundtrip() {
    let hash = argon2id::hash("пароль日本語🔒");
    assert!(argon2id::verify("пароль日本語🔒", &hash));
    assert!(!argon2id::verify("пароль", &hash));
}

#[test]
fn empty_password_roundtrip() {
    let hash = argon2id::hash("");
    assert!(argon2id::verify("", &hash));
    assert!(!argon2id::verify("x", &hash));
}
