//! HAP secure session transport encryption (HomeKit Accessory Protocol R2
//! §6.5.2 "Session Security").
//!
//! Once a session key has been established (via Pair Verify) each side derives
//! two ChaCha20-Poly1305 keys with `HKDF-SHA-512` and encrypts the HTTP stream
//! in frames of at most [`MAX_FRAME_LEN`] plaintext bytes. Each frame is:
//!
//! ```text
//! | 2-byte little-endian length (AAD) | ciphertext | 16-byte Poly1305 tag |
//! ```
//!
//! The nonce is four zero bytes followed by a little-endian 64-bit frame
//! counter that starts at zero and increments per frame, independently for each
//! direction.

use crate::chacha::{self, KEY_LEN, TAG_LEN};
use crate::error::{HapError, Result};
use crate::hkdf::hkdf_sha512_32;

/// Maximum plaintext length of a single HAP session frame.
pub const MAX_FRAME_LEN: usize = 1024;

const CONTROL_SALT: &[u8] = b"Control-Salt";
const READ_INFO: &[u8] = b"Control-Read-Encryption-Key";
const WRITE_INFO: &[u8] = b"Control-Write-Encryption-Key";

/// A directional ChaCha20-Poly1305 key together with its frame counter.
struct Direction {
    key: [u8; KEY_LEN],
    counter: u64,
}

impl Direction {
    fn nonce(&self) -> [u8; 12] {
        let mut suffix = [0u8; 8];
        suffix.copy_from_slice(&self.counter.to_le_bytes());
        chacha::nonce(&suffix)
    }
}

/// An established HAP secure session for one peer (accessory or controller).
///
/// The `send` direction is used by [`HapSession::encrypt`] and the `recv`
/// direction by [`HapSession::decrypt`]; the two peers are mirror images of
/// each other.
pub struct HapSession {
    send: Direction,
    recv: Direction,
}

impl HapSession {
    /// Build the accessory (server) side of a session from the Pair Verify
    /// shared secret. The accessory sends with the *read* key and receives with
    /// the *write* key.
    pub fn accessory(shared_secret: &[u8]) -> Result<Self> {
        let read = hkdf_sha512_32(CONTROL_SALT, shared_secret, READ_INFO)?;
        let write = hkdf_sha512_32(CONTROL_SALT, shared_secret, WRITE_INFO)?;
        Ok(Self::from_keys(read, write))
    }

    /// Build the controller (client) side of a session from the Pair Verify
    /// shared secret. The controller sends with the *write* key and receives
    /// with the *read* key.
    pub fn controller(shared_secret: &[u8]) -> Result<Self> {
        let read = hkdf_sha512_32(CONTROL_SALT, shared_secret, READ_INFO)?;
        let write = hkdf_sha512_32(CONTROL_SALT, shared_secret, WRITE_INFO)?;
        Ok(Self::from_keys(write, read))
    }

    fn from_keys(send_key: [u8; KEY_LEN], recv_key: [u8; KEY_LEN]) -> Self {
        HapSession {
            send: Direction {
                key: send_key,
                counter: 0,
            },
            recv: Direction {
                key: recv_key,
                counter: 0,
            },
        }
    }

    /// Encrypt `plaintext`, splitting it into HAP frames and advancing the send
    /// counter once per frame. An empty input produces a single empty frame,
    /// matching the reference implementation.
    pub fn encrypt(&mut self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let mut out = Vec::new();
        let mut chunks = plaintext.chunks(MAX_FRAME_LEN);
        // `chunks` yields nothing for an empty slice; emit a single empty frame.
        let mut wrote_any = false;
        for chunk in &mut chunks {
            self.encrypt_frame(chunk, &mut out)?;
            wrote_any = true;
        }
        if !wrote_any {
            self.encrypt_frame(&[], &mut out)?;
        }
        Ok(out)
    }

    fn encrypt_frame(&mut self, chunk: &[u8], out: &mut Vec<u8>) -> Result<()> {
        let len = chunk.len() as u16;
        let aad = len.to_le_bytes();
        let frame = chacha::seal(&self.send.key, &self.send.nonce(), &aad, chunk)?;
        self.send.counter = self.send.counter.wrapping_add(1);
        out.extend_from_slice(&aad);
        out.extend_from_slice(&frame);
        Ok(())
    }

    /// Decrypt a stream of HAP frames produced by the peer, advancing the
    /// receive counter once per frame. Returns [`HapError::Decrypt`] on a
    /// truncated or inauthentic stream.
    pub fn decrypt(&mut self, mut data: &[u8]) -> Result<Vec<u8>> {
        let mut out = Vec::new();
        while !data.is_empty() {
            if data.len() < 2 {
                return Err(HapError::Decrypt);
            }
            let len = u16::from_le_bytes([data[0], data[1]]) as usize;
            let frame_end = 2 + len + TAG_LEN;
            if data.len() < frame_end {
                return Err(HapError::Decrypt);
            }
            let aad = &data[0..2];
            let body = &data[2..frame_end];
            let plain = chacha::open(&self.recv.key, &self.recv.nonce(), aad, body)?;
            self.recv.counter = self.recv.counter.wrapping_add(1);
            out.extend_from_slice(&plain);
            data = &data[frame_end..];
        }
        Ok(out)
    }
}
