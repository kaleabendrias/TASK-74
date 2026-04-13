use tourism_backend::crypto::{csrf, sha256};

#[test]
fn csrf_token_length() {
    let token = csrf::generate_token();
    // 32 bytes base64url-no-pad → 43 characters
    assert_eq!(token.len(), 43);
}

#[test]
fn csrf_tokens_are_unique() {
    let t1 = csrf::generate_token();
    let t2 = csrf::generate_token();
    assert_ne!(t1, t2);
}

#[test]
fn csrf_token_is_url_safe() {
    let token = csrf::generate_token();
    assert!(token.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_'));
}

#[test]
fn sha256_known_vector() {
    // SHA-256 of empty string
    let hash = sha256::hash_bytes(b"");
    assert_eq!(hash, "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
}

#[test]
fn sha256_hello_world() {
    let hash = sha256::hash_bytes(b"hello world");
    assert_eq!(hash, "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9");
}

#[test]
fn sha256_deterministic() {
    let h1 = sha256::hash_bytes(b"test data");
    let h2 = sha256::hash_bytes(b"test data");
    assert_eq!(h1, h2);
}

#[test]
fn sha256_different_inputs_different_hashes() {
    let h1 = sha256::hash_bytes(b"input1");
    let h2 = sha256::hash_bytes(b"input2");
    assert_ne!(h1, h2);
}
