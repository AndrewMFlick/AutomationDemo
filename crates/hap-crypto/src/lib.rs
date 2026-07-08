//! Security-critical primitives for the HAP port.
//!
//! Ported from the Apache-2.0 HAP-NodeJS reference (`src/lib/util/hapCrypto.ts`).
//! This first slice covers per-frame nonce derivation and a constant-time tag
//! comparison used by the encrypted-session transport.

/// Errors returned by the crypto layer. No panics in library code.
#[derive(Debug, PartialEq, Eq)]
pub enum HapError {
    /// A key or buffer had an unexpected length.
    InvalidLength,
}

/// Derive the 96-bit ChaCha20-Poly1305 nonce for a HAP session frame.
///
/// HAP uses a 64-bit little-endian frame counter placed in the low 8 bytes of a
/// 12-byte nonce, with the high 4 bytes left as zero. Each frame MUST use a
/// fresh counter so a nonce is never reused under the same key.
pub fn derive_nonce(counter: u64) -> [u8; 12] {
    let mut nonce = [0u8; 12];
    nonce[4..].copy_from_slice(&counter.to_le_bytes());
    nonce
}

/// Constant-time comparison of two byte slices (e.g. Poly1305 tags).
///
/// Returns `false` for length mismatches and never short-circuits on the first
/// differing byte, to avoid leaking tag contents through timing.
pub fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nonce_places_counter_in_low_bytes() {
        assert_eq!(derive_nonce(1), [0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn distinct_counters_give_distinct_nonces() {
        assert_ne!(derive_nonce(1), derive_nonce(2));
    }

    #[test]
    fn ct_eq_matches_and_rejects() {
        assert!(ct_eq(b"abcd", b"abcd"));
        assert!(!ct_eq(b"abcd", b"abce"));
        assert!(!ct_eq(b"abc", b"abcd"));
    }
}
