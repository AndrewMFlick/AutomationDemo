//! Error type for the `hap-crypto` crate.
//!
//! Library code in this crate never panics on untrusted input and never calls
//! `unwrap()`; all fallible operations return [`HapError`].

use core::fmt;

/// Errors produced by the HAP crypto primitives.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum HapError {
    /// A supplied buffer had an unexpected length.
    InvalidLength {
        /// Human readable name of the offending parameter.
        what: &'static str,
    },
    /// An SRP public key was out of range (`A` or `B` not in `1..N-1`).
    InvalidPublicKey,
    /// The peer's SRP proof did not match (wrong setup code).
    BadProof,
    /// HKDF expansion failed (requested output too large).
    Hkdf,
    /// ChaCha20-Poly1305 authentication failed while decrypting.
    Decrypt,
    /// ChaCha20-Poly1305 encryption failed.
    Encrypt,
    /// Failed to gather secure random bytes from the operating system.
    Rng,
}

impl fmt::Display for HapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HapError::InvalidLength { what } => write!(f, "invalid length for {what}"),
            HapError::InvalidPublicKey => f.write_str("SRP public key out of range"),
            HapError::BadProof => f.write_str("SRP proof verification failed"),
            HapError::Hkdf => f.write_str("HKDF expansion failed"),
            HapError::Decrypt => f.write_str("AEAD decryption/authentication failed"),
            HapError::Encrypt => f.write_str("AEAD encryption failed"),
            HapError::Rng => f.write_str("failed to obtain secure random bytes"),
        }
    }
}

impl std::error::Error for HapError {}

/// Convenience alias for results returned by this crate.
pub type Result<T> = core::result::Result<T, HapError>;
