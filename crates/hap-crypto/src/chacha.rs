//! ChaCha20-Poly1305 AEAD as used by HAP (HomeKit Accessory Protocol R2).
//!
//! HAP always uses a 96-bit (12-byte) nonce whose leading 4 bytes are zero.
//! During Pair Setup / Pair Verify the trailing 8 bytes are an ASCII label
//! such as `PS-Msg05`; for an established session they are a little-endian
//! message counter (see [`crate::session`]).

use chacha20poly1305::aead::{Aead, Payload};
use chacha20poly1305::{ChaCha20Poly1305, Key, KeyInit, Nonce};

use crate::error::{HapError, Result};

/// Length in bytes of a ChaCha20-Poly1305 key.
pub const KEY_LEN: usize = 32;
/// Length in bytes of the Poly1305 authentication tag.
pub const TAG_LEN: usize = 16;

/// Build the 12-byte HAP nonce from an 8-byte suffix (4 leading zero bytes).
pub fn nonce(suffix: &[u8; 8]) -> [u8; 12] {
    let mut n = [0u8; 12];
    n[4..].copy_from_slice(suffix);
    n
}

/// Encrypt `plaintext` with the given `key`, `nonce` and additional
/// authenticated data `aad`, returning `ciphertext || tag`.
pub fn seal(
    key: &[u8; KEY_LEN],
    nonce12: &[u8; 12],
    aad: &[u8],
    plaintext: &[u8],
) -> Result<Vec<u8>> {
    let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
    cipher
        .encrypt(
            Nonce::from_slice(nonce12),
            Payload {
                msg: plaintext,
                aad,
            },
        )
        .map_err(|_| HapError::Encrypt)
}

/// Decrypt and authenticate `ciphertext || tag`, returning the plaintext.
///
/// Returns [`HapError::Decrypt`] if authentication fails.
pub fn open(
    key: &[u8; KEY_LEN],
    nonce12: &[u8; 12],
    aad: &[u8],
    ciphertext: &[u8],
) -> Result<Vec<u8>> {
    if ciphertext.len() < TAG_LEN {
        return Err(HapError::Decrypt);
    }
    let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
    cipher
        .decrypt(
            Nonce::from_slice(nonce12),
            Payload {
                msg: ciphertext,
                aad,
            },
        )
        .map_err(|_| HapError::Decrypt)
}
