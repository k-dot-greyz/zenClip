use sha2::{Digest, Sha256};

/// Hex SHA256 of the *sanitized PNG bytes* (content-addressed output).
pub fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::sha256_hex;

    #[test]
    fn known_empty_digest() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn different_bytes_different_hash() {
        assert_ne!(sha256_hex(b"a"), sha256_hex(b"b"));
    }
}
