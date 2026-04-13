use sha2::{Digest, Sha256};

use super::hmac_sign::hex_encode;

pub fn hash_bytes(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex_encode(&hasher.finalize())
}
