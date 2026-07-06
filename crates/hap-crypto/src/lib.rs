//! `hap-crypto` — cryptographic primitives for the Rust port of HAP-NodeJS.
//!
//! This crate implements the HomeKit Accessory Protocol (HAP R2) Pair Setup
//! cryptography and secure session transport:
//!
//! * [`srp`] — SRP-6a Pair Setup (RFC 5054 3072-bit group, SHA-512), byte-for-byte
//!   compatible with HAP-NodeJS (`fast-srp-hap`) and the iOS Home app.
//! * [`hkdf`] — `HKDF-SHA-512` key derivation.
//! * [`chacha`] — ChaCha20-Poly1305 AEAD helpers.
//! * [`pair_setup`] — the M5/M6 encrypted exchange key derivation.
//! * [`session`] — ChaCha20-Poly1305 secure session transport framing.
//!
//! HAP spec sections implemented: §5.6 (Pair Setup) and §6.5.2 (Session
//! Security).

#![forbid(unsafe_code)]

pub mod chacha;
pub mod error;
pub mod hkdf;
pub mod pair_setup;
pub mod session;
pub mod srp;

pub use error::{HapError, Result};
pub use session::HapSession;
pub use srp::{PairSetupOutcome, PairSetupServer};
