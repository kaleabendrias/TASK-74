use sha2::{Digest, Sha256};

use super::hmac_sign::hex_encode;

/// Computes a SHA-256 digest of the given bytes and returns it as a hex string.
pub fn hash_bytes(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex_encode(&hasher.finalize())
}
