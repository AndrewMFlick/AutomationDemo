//! HAP Pair Setup encrypted exchange (HomeKit Accessory Protocol R2 §5.6.5 /
//! §5.6.6).
//!
//! After SRP establishes the shared session key `K`, the M5/M6 sub-TLVs are
//! exchanged encrypted with ChaCha20-Poly1305 under a key derived from `K` with
//! `HKDF-SHA-512`. The same `K` is also used to derive the material each side
//! signs with its long-term Ed25519 key.

use crate::chacha::{self};
use crate::error::Result;
use crate::hkdf::hkdf_sha512_32;

const ENCRYPT_SALT: &[u8] = b"Pair-Setup-Encrypt-Salt";
const ENCRYPT_INFO: &[u8] = b"Pair-Setup-Encrypt-Info";
const CONTROLLER_SIGN_SALT: &[u8] = b"Pair-Setup-Controller-Sign-Salt";
const CONTROLLER_SIGN_INFO: &[u8] = b"Pair-Setup-Controller-Sign-Info";
const ACCESSORY_SIGN_SALT: &[u8] = b"Pair-Setup-Accessory-Sign-Salt";
const ACCESSORY_SIGN_INFO: &[u8] = b"Pair-Setup-Accessory-Sign-Info";

/// AEAD nonce suffix for the controller's M5 payload.
const NONCE_MSG05: &[u8; 8] = b"PS-Msg05";
/// AEAD nonce suffix for the accessory's M6 payload.
const NONCE_MSG06: &[u8; 8] = b"PS-Msg06";

/// Derive the ChaCha20-Poly1305 key protecting the M5/M6 sub-TLVs from the SRP
/// session key `K`.
pub fn encrypt_key(session_key: &[u8]) -> Result<[u8; 32]> {
    hkdf_sha512_32(ENCRYPT_SALT, session_key, ENCRYPT_INFO)
}

/// Derive the material the controller signs with its Ed25519 long-term key.
pub fn controller_sign_material(session_key: &[u8]) -> Result<[u8; 32]> {
    hkdf_sha512_32(CONTROLLER_SIGN_SALT, session_key, CONTROLLER_SIGN_INFO)
}

/// Derive the material the accessory signs with its Ed25519 long-term key.
pub fn accessory_sign_material(session_key: &[u8]) -> Result<[u8; 32]> {
    hkdf_sha512_32(ACCESSORY_SIGN_SALT, session_key, ACCESSORY_SIGN_INFO)
}

/// Decrypt the controller's M5 encrypted sub-TLV using the SRP session key.
pub fn decrypt_m5(session_key: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>> {
    let key = encrypt_key(session_key)?;
    chacha::open(&key, &chacha::nonce(NONCE_MSG05), &[], ciphertext)
}

/// Encrypt the accessory's M6 sub-TLV response using the SRP session key.
pub fn encrypt_m6(session_key: &[u8], plaintext: &[u8]) -> Result<Vec<u8>> {
    let key = encrypt_key(session_key)?;
    chacha::seal(&key, &chacha::nonce(NONCE_MSG06), &[], plaintext)
}
