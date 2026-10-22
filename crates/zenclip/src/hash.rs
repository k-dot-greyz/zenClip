use sha2::{Digest, Sha256};

use crate::error::{Result, ZenclipError};

/// SHA-256 hex digest length (32 bytes → 64 lowercase hex chars).
pub const SHA256_HEX_LEN: usize = 64;

/// Hex SHA256 of the *sanitized PNG bytes* (content-addressed output).
pub fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

/// True iff `s` is exactly 64 lowercase `0-9a-f` characters.
///
/// Inbox filenames are `{sha256}.png`. Anything else (uppercase, `/`, `..`,
/// extra length) must not be joined onto the inbox path.
pub fn is_sha256_hex(s: &str) -> bool {
    s.len() == SHA256_HEX_LEN && s.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

/// Reject values that are not a lowercase SHA-256 hex digest.
pub fn require_sha256_hex(s: &str) -> Result<&str> {
    if is_sha256_hex(s) {
        Ok(s)
    } else {
        Err(ZenclipError::InvalidSha256 {
            value: s.to_string(),
        })
    }
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

    #[test]
    fn sha256_hex_is_64_lowercase() {
        let digest = sha256_hex(b"");
        assert!(super::is_sha256_hex(&digest), "{digest}");
        assert_eq!(digest.len(), super::SHA256_HEX_LEN);
        assert_eq!(digest, digest.to_ascii_lowercase());
    }

    #[test]
    fn require_sha256_hex_rejects_traversal_and_non_hex() {
        for bad in [
            "../escaped",
            "not-a-hash",
            "",
            "gggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggg",
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b85",
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855ff",
        ] {
            let err = super::require_sha256_hex(bad).unwrap_err();
            match err {
                crate::error::ZenclipError::InvalidSha256 { value } => {
                    assert_eq!(value, bad);
                }
                other => panic!("unexpected {other} for {bad:?}"),
            }
        }
        assert!(super::require_sha256_hex(
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        )
        .is_ok());
    }
}
