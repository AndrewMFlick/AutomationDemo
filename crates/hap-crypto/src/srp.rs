//! SRP-6a Pair Setup for the HomeKit Accessory Protocol (HAP R2 §5.6).
//!
//! HAP uses SRP-6a with the RFC 5054 3072-bit group (`g = 5`) and SHA-512.
//! The proof values follow the RFC 5054 layout used by Apple / HAP-NodeJS:
//!
//! ```text
//! k  = H(PAD(N) | PAD(g))
//! x  = H(s | H(I | ":" | P))
//! v  = g^x mod N
//! B  = (k*v + g^b) mod N
//! u  = H(PAD(A) | PAD(B))
//! S  = (A * v^u)^b mod N
//! K  = H(S)
//! M1 = H(H(N) XOR H(g) | H(I) | s | A | B | K)
//! M2 = H(A | M1 | K)
//! ```
//!
//! `I` is the fixed identity `Pair-Setup` and `P` is the accessory setup code
//! formatted `XXX-XX-XXX`. This implementation is byte-for-byte compatible with
//! HAP-NodeJS (`fast-srp-hap`) and therefore with the iOS Home app.

use num_bigint::BigUint;
use sha2::{Digest, Sha512};
use subtle::ConstantTimeEq;

use crate::error::{HapError, Result};

/// Length in bytes of the SRP modulus `N` (3072 bits).
pub const N_LEN: usize = 384;
/// Length in bytes of the salt used during Pair Setup.
pub const SALT_LEN: usize = 16;
/// Length in bytes of an SRP proof / session key (SHA-512 output).
pub const PROOF_LEN: usize = 64;

/// The fixed SRP identity (`I`) used for HAP Pair Setup.
pub const IDENTITY: &[u8] = b"Pair-Setup";

/// The RFC 5054 3072-bit group generator, `g = 5`.
const G: &[u8] = &[5];

/// The RFC 5054 3072-bit safe prime `N`, big-endian.
const N_BYTES: [u8; N_LEN] = [
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xc9, 0x0f, 0xda, 0xa2, 0x21, 0x68, 0xc2, 0x34,
    0xc4, 0xc6, 0x62, 0x8b, 0x80, 0xdc, 0x1c, 0xd1, 0x29, 0x02, 0x4e, 0x08, 0x8a, 0x67, 0xcc, 0x74,
    0x02, 0x0b, 0xbe, 0xa6, 0x3b, 0x13, 0x9b, 0x22, 0x51, 0x4a, 0x08, 0x79, 0x8e, 0x34, 0x04, 0xdd,
    0xef, 0x95, 0x19, 0xb3, 0xcd, 0x3a, 0x43, 0x1b, 0x30, 0x2b, 0x0a, 0x6d, 0xf2, 0x5f, 0x14, 0x37,
    0x4f, 0xe1, 0x35, 0x6d, 0x6d, 0x51, 0xc2, 0x45, 0xe4, 0x85, 0xb5, 0x76, 0x62, 0x5e, 0x7e, 0xc6,
    0xf4, 0x4c, 0x42, 0xe9, 0xa6, 0x37, 0xed, 0x6b, 0x0b, 0xff, 0x5c, 0xb6, 0xf4, 0x06, 0xb7, 0xed,
    0xee, 0x38, 0x6b, 0xfb, 0x5a, 0x89, 0x9f, 0xa5, 0xae, 0x9f, 0x24, 0x11, 0x7c, 0x4b, 0x1f, 0xe6,
    0x49, 0x28, 0x66, 0x51, 0xec, 0xe4, 0x5b, 0x3d, 0xc2, 0x00, 0x7c, 0xb8, 0xa1, 0x63, 0xbf, 0x05,
    0x98, 0xda, 0x48, 0x36, 0x1c, 0x55, 0xd3, 0x9a, 0x69, 0x16, 0x3f, 0xa8, 0xfd, 0x24, 0xcf, 0x5f,
    0x83, 0x65, 0x5d, 0x23, 0xdc, 0xa3, 0xad, 0x96, 0x1c, 0x62, 0xf3, 0x56, 0x20, 0x85, 0x52, 0xbb,
    0x9e, 0xd5, 0x29, 0x07, 0x70, 0x96, 0x96, 0x6d, 0x67, 0x0c, 0x35, 0x4e, 0x4a, 0xbc, 0x98, 0x04,
    0xf1, 0x74, 0x6c, 0x08, 0xca, 0x18, 0x21, 0x7c, 0x32, 0x90, 0x5e, 0x46, 0x2e, 0x36, 0xce, 0x3b,
    0xe3, 0x9e, 0x77, 0x2c, 0x18, 0x0e, 0x86, 0x03, 0x9b, 0x27, 0x83, 0xa2, 0xec, 0x07, 0xa2, 0x8f,
    0xb5, 0xc5, 0x5d, 0xf0, 0x6f, 0x4c, 0x52, 0xc9, 0xde, 0x2b, 0xcb, 0xf6, 0x95, 0x58, 0x17, 0x18,
    0x39, 0x95, 0x49, 0x7c, 0xea, 0x95, 0x6a, 0xe5, 0x15, 0xd2, 0x26, 0x18, 0x98, 0xfa, 0x05, 0x10,
    0x15, 0x72, 0x8e, 0x5a, 0x8a, 0xaa, 0xc4, 0x2d, 0xad, 0x33, 0x17, 0x0d, 0x04, 0x50, 0x7a, 0x33,
    0xa8, 0x55, 0x21, 0xab, 0xdf, 0x1c, 0xba, 0x64, 0xec, 0xfb, 0x85, 0x04, 0x58, 0xdb, 0xef, 0x0a,
    0x8a, 0xea, 0x71, 0x57, 0x5d, 0x06, 0x0c, 0x7d, 0xb3, 0x97, 0x0f, 0x85, 0xa6, 0xe1, 0xe4, 0xc7,
    0xab, 0xf5, 0xae, 0x8c, 0xdb, 0x09, 0x33, 0xd7, 0x1e, 0x8c, 0x94, 0xe0, 0x4a, 0x25, 0x61, 0x9d,
    0xce, 0xe3, 0xd2, 0x26, 0x1a, 0xd2, 0xee, 0x6b, 0xf1, 0x2f, 0xfa, 0x06, 0xd9, 0x8a, 0x08, 0x64,
    0xd8, 0x76, 0x02, 0x73, 0x3e, 0xc8, 0x6a, 0x64, 0x52, 0x1f, 0x2b, 0x18, 0x17, 0x7b, 0x20, 0x0c,
    0xbb, 0xe1, 0x17, 0x57, 0x7a, 0x61, 0x5d, 0x6c, 0x77, 0x09, 0x88, 0xc0, 0xba, 0xd9, 0x46, 0xe2,
    0x08, 0xe2, 0x4f, 0xa0, 0x74, 0xe5, 0xab, 0x31, 0x43, 0xdb, 0x5b, 0xfc, 0xe0, 0xfd, 0x10, 0x8e,
    0x4b, 0x82, 0xd1, 0x20, 0xa9, 0x3a, 0xd2, 0xca, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
];

fn modulus() -> BigUint {
    BigUint::from_bytes_be(&N_BYTES)
}

fn generator() -> BigUint {
    BigUint::from_bytes_be(G)
}

fn sha512(parts: &[&[u8]]) -> [u8; PROOF_LEN] {
    let mut h = Sha512::new();
    for p in parts {
        h.update(p);
    }
    h.finalize().into()
}

/// Left-pad a big-endian byte string to exactly [`N_LEN`] bytes (`PAD()`).
fn pad_to_n(bytes: &[u8]) -> [u8; N_LEN] {
    let mut out = [0u8; N_LEN];
    let start = N_LEN - bytes.len();
    out[start..].copy_from_slice(bytes);
    out
}

/// Encode a [`BigUint`] as a fixed-width [`N_LEN`]-byte big-endian buffer.
fn to_n_bytes(v: &BigUint) -> [u8; N_LEN] {
    pad_to_n(&v.to_bytes_be())
}

/// The SRP-6a multiplier `k = H(PAD(N) | PAD(g))`.
fn compute_k() -> BigUint {
    let k = sha512(&[&N_BYTES, &pad_to_n(G)]);
    BigUint::from_bytes_be(&k)
}

/// The SRP private key `x = H(s | H(I | ":" | P))`.
fn compute_x(salt: &[u8], identity: &[u8], password: &[u8]) -> BigUint {
    let inner = sha512(&[identity, b":", password]);
    let x = sha512(&[salt, &inner]);
    BigUint::from_bytes_be(&x)
}

/// Compute the SRP verifier `v = g^x mod N` for the given credentials.
///
/// This is what an accessory persists (alongside the salt) so it never needs to
/// store the setup code itself.
pub fn compute_verifier(salt: &[u8], password: &[u8]) -> [u8; N_LEN] {
    let x = compute_x(salt, IDENTITY, password);
    to_n_bytes(&generator().modpow(&x, &modulus()))
}

/// Result of a successful [`PairSetupServer::verify`] call.
#[derive(Clone)]
pub struct PairSetupOutcome {
    /// The server proof `M2` to send back to the controller.
    pub server_proof: [u8; PROOF_LEN],
    /// The shared SRP session key `K` (input keying material for HKDF).
    pub session_key: [u8; PROOF_LEN],
}

/// The accessory (server) side of HAP SRP Pair Setup.
pub struct PairSetupServer {
    salt: [u8; SALT_LEN],
    b_priv: BigUint,
    verifier: BigUint,
    b_pub: [u8; N_LEN],
}

impl PairSetupServer {
    /// Create a new session for `setup_code` (formatted `XXX-XX-XXX`), sampling
    /// a fresh random salt and private key from the operating system RNG.
    pub fn new(setup_code: &str) -> Result<Self> {
        let mut salt = [0u8; SALT_LEN];
        let mut secret = [0u8; 32];
        getrandom::getrandom(&mut salt).map_err(|_| HapError::Rng)?;
        getrandom::getrandom(&mut secret).map_err(|_| HapError::Rng)?;
        Ok(Self::with_secrets(setup_code.as_bytes(), salt, &secret))
    }

    /// Create a session from an explicit salt and server private key `b`.
    ///
    /// Intended for deterministic tests and interop vectors; production code
    /// should prefer [`PairSetupServer::new`].
    pub fn with_secrets(password: &[u8], salt: [u8; SALT_LEN], b_priv: &[u8]) -> Self {
        let n = modulus();
        let g = generator();
        let x = compute_x(&salt, IDENTITY, password);
        let verifier = g.modpow(&x, &n);
        let b_priv = BigUint::from_bytes_be(b_priv);
        // B = (k*v + g^b) mod N
        let b_pub_num = (compute_k() * &verifier + g.modpow(&b_priv, &n)) % &n;
        Self {
            salt,
            b_priv,
            verifier,
            b_pub: to_n_bytes(&b_pub_num),
        }
    }

    /// The 16-byte salt to send to the controller (`kTLVType_Salt`).
    pub fn salt(&self) -> &[u8; SALT_LEN] {
        &self.salt
    }

    /// The accessory public key `B` to send to the controller
    /// (`kTLVType_PublicKey`), fixed at [`N_LEN`] bytes.
    pub fn public_key(&self) -> &[u8; N_LEN] {
        &self.b_pub
    }

    /// Verify the controller public key `A` and proof `M1`.
    ///
    /// On success returns the server proof `M2` and the shared session key `K`.
    /// Returns [`HapError::InvalidPublicKey`] if `A` is not in `1..N-1`, or
    /// [`HapError::BadProof`] if the controller proof does not match (wrong
    /// setup code).
    pub fn verify(&self, client_public: &[u8], client_proof: &[u8]) -> Result<PairSetupOutcome> {
        let n = modulus();
        let a = BigUint::from_bytes_be(client_public);
        // A must not be zero mod N.
        if a.is_zero_mod(&n) {
            return Err(HapError::InvalidPublicKey);
        }

        // u = H(PAD(A) | PAD(B))
        let u = BigUint::from_bytes_be(&sha512(&[&pad_to_n(client_public), &self.b_pub]));
        // S = (A * v^u)^b mod N
        let s = (&a * self.verifier.modpow(&u, &n)).modpow(&self.b_priv, &n);
        let s_bytes = to_n_bytes(&s);
        // K = H(S)
        let session_key = sha512(&[&s_bytes]);

        let expected_m1 = compute_m1(
            client_public,
            &self.b_pub,
            self.salt.as_slice(),
            &session_key,
        );
        if expected_m1.ct_eq(&proof_from(client_proof)?).unwrap_u8() != 1 {
            return Err(HapError::BadProof);
        }

        let server_proof = compute_m2(client_public, &expected_m1, &session_key);
        Ok(PairSetupOutcome {
            server_proof,
            session_key,
        })
    }
}

/// `M1 = H(H(N) XOR H(g) | H(I) | s | A | B | K)`.
fn compute_m1(a: &[u8], b: &[u8], salt: &[u8], key: &[u8]) -> [u8; PROOF_LEN] {
    let mut h_n = sha512(&[&N_BYTES]);
    let h_g = sha512(&[G]);
    for (dst, src) in h_n.iter_mut().zip(h_g.iter()) {
        *dst ^= *src;
    }
    let h_i = sha512(&[IDENTITY]);
    sha512(&[&h_n, &h_i, salt, a, b, key])
}

/// `M2 = H(A | M1 | K)`.
fn compute_m2(a: &[u8], m1: &[u8], key: &[u8]) -> [u8; PROOF_LEN] {
    sha512(&[a, m1, key])
}

fn proof_from(proof: &[u8]) -> Result<[u8; PROOF_LEN]> {
    proof
        .try_into()
        .map_err(|_| HapError::InvalidLength { what: "SRP proof" })
}

trait ZeroModN {
    fn is_zero_mod(&self, n: &BigUint) -> bool;
}

impl ZeroModN for BigUint {
    fn is_zero_mod(&self, n: &BigUint) -> bool {
        use num_bigint::BigUint as B;
        (self % n) == B::from(0u8)
    }
}
