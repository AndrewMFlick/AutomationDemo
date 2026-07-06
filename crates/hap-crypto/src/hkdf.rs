//! HKDF-SHA-512 key derivation as used throughout HAP (HomeKit Accessory
//! Protocol R2 §5.6 "Pair Setup" and §6.5.2 "Session Security").

use hkdf::Hkdf;
use sha2::Sha512;

use crate::error::{HapError, Result};

/// Derive `out.len()` bytes with `HKDF-SHA-512(salt, ikm, info)`.
///
/// Both `salt` and `info` are the ASCII label strings defined by the HAP
/// specification (for example `Pair-Setup-Encrypt-Salt` /
/// `Pair-Setup-Encrypt-Info`). Returns [`HapError::Hkdf`] only if `out` is
/// longer than `255 * 64` bytes, which HAP never requests.
pub fn hkdf_sha512(salt: &[u8], ikm: &[u8], info: &[u8], out: &mut [u8]) -> Result<()> {
    let hk = Hkdf::<Sha512>::new(Some(salt), ikm);
    hk.expand(info, out).map_err(|_| HapError::Hkdf)
}

/// Derive a 32-byte key with `HKDF-SHA-512`, the common case for HAP AEAD keys.
pub fn hkdf_sha512_32(salt: &[u8], ikm: &[u8], info: &[u8]) -> Result<[u8; 32]> {
    let mut out = [0u8; 32];
    hkdf_sha512(salt, ikm, info, &mut out)?;
    Ok(out)
}
