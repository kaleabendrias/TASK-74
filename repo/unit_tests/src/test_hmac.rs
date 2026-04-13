use tourism_backend::crypto::hmac_sign;

#[test]
fn sign_produces_consistent_output() {
    let sig1 = hmac_sign::sign("secret", "message");
    let sig2 = hmac_sign::sign("secret", "message");
    assert_eq!(sig1, sig2);
}

#[test]
fn different_messages_different_signatures() {
    let s1 = hmac_sign::sign("secret", "message1");
    let s2 = hmac_sign::sign("secret", "message2");
    assert_ne!(s1, s2);
}

#[test]
fn different_keys_different_signatures() {
    let s1 = hmac_sign::sign("key1", "message");
    let s2 = hmac_sign::sign("key2", "message");
    assert_ne!(s1, s2);
}

#[test]
fn verify_valid_signature() {
    let sig = hmac_sign::sign("my-secret", "payload");
    assert!(hmac_sign::verify_signature("my-secret", "payload", &sig));
}

#[test]
fn verify_invalid_signature_fails() {
    assert!(!hmac_sign::verify_signature("my-secret", "payload", "badhex"));
}

#[test]
fn verify_tampered_signature_fails() {
    let mut sig = hmac_sign::sign("my-secret", "payload");
    // Flip last hex char
    let last = sig.pop().unwrap();
    let replacement = if last == '0' { '1' } else { '0' };
    sig.push(replacement);
    assert!(!hmac_sign::verify_signature("my-secret", "payload", &sig));
}

#[test]
fn verify_wrong_key_fails() {
    let sig = hmac_sign::sign("key1", "payload");
    assert!(!hmac_sign::verify_signature("key2", "payload", &sig));
}

#[test]
fn verify_wrong_message_fails() {
    let sig = hmac_sign::sign("key", "msg1");
    assert!(!hmac_sign::verify_signature("key", "msg2", &sig));
}

#[test]
fn hex_encode_decode_roundtrip() {
    let data = vec![0x00, 0x01, 0x0A, 0xFF, 0xDE, 0xAD];
    let encoded = hmac_sign::hex_encode(&data);
    assert_eq!(encoded, "00010affDEAD".to_lowercase());
    let decoded = hmac_sign::hex_decode(&encoded).unwrap();
    assert_eq!(decoded, data);
}

#[test]
fn hex_decode_odd_length_returns_none() {
    assert!(hmac_sign::hex_decode("abc").is_none());
}

#[test]
fn hex_decode_invalid_chars_returns_none() {
    assert!(hmac_sign::hex_decode("zzzz").is_none());
}

#[test]
fn empty_message_and_key() {
    let sig = hmac_sign::sign("", "");
    assert!(hmac_sign::verify_signature("", "", &sig));
}

#[test]
fn signature_is_lowercase_hex() {
    let sig = hmac_sign::sign("k", "m");
    assert!(sig.chars().all(|c| c.is_ascii_hexdigit() && !c.is_uppercase()));
}

#[test]
fn request_signing_simulation() {
    let key = "req-sign-key-tourism-portal-2024";
    let body = r#"{"entity_type":"resource","data":{}}"#;
    let nonce = "unique-nonce-123";
    let timestamp = "1700000000";
    let message = format!("{}{}{}", body, nonce, timestamp);
    let sig = hmac_sign::sign(key, &message);
    assert!(hmac_sign::verify_signature(key, &message, &sig));
    // Tamper with timestamp
    let tampered = format!("{}{}{}", body, nonce, "1700000001");
    assert!(!hmac_sign::verify_signature(key, &tampered, &sig));
}
