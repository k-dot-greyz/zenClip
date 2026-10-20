use sha2::{Digest, Sha256};

/// Hex SHA256 of the *sanitized PNG bytes* (content-addressed output).
pub fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

/// Content-addressed inbox filenames must be exactly 64 ASCII hex digits.
pub fn is_valid_sha256_hex(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b) || (b'A'..=b'F').contains(&b))
}

#[cfg(test)]
mod tests {
    use super::{is_valid_sha256_hex, sha256_hex};

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

    #[test]
    fn validates_hex_content_hash_shape() {
        assert!(is_valid_sha256_hex(&sha256_hex(b"zenclip")));
        assert!(!is_valid_sha256_hex("not-a-hash"));
        assert!(!is_valid_sha256_hex(
            "../../../../tmp/evil00000000000000000000000000000000000000000000000000"
        ));
    }
}
