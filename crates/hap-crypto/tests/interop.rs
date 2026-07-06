//! Byte-exact interop tests for HAP Pair Setup + secure session.
//!
//! All expected values were generated from HAP-NodeJS's `fast-srp-hap` (SRP,
//! §5.6) and Node.js `crypto` (HKDF-SHA-512 and ChaCha20-Poly1305), so passing
//! these tests demonstrates wire compatibility with HAP-NodeJS and the iOS Home
//! app.

use hap_crypto::error::HapError;
use hap_crypto::{chacha, hkdf, pair_setup, session, srp};

fn hex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).expect("valid hex"))
        .collect()
}

fn salt16() -> [u8; 16] {
    hex(SALT).try_into().expect("16-byte salt")
}

// --- Fixed interop vectors (see PR description / HAP-NodeJS generator) ---
const SALT: &str = "0102030405060708090a0b0c0d0e0f10";
const PASSWORD: &[u8] = b"123-45-678";
// Server private key b = 0xBB * 32, client private key a = 0xAA * 32.
const B_PRIV: [u8; 32] = [0xBB; 32];
const V: &str = "b07f982bc93f5cdd838093f85e8afa018035a0a2898385256bd702a78b5a6c7a8aee865341b1914e26c29e34da5e21dec283a7ed97e358a6efb6e27c0a0a2129a5150f7efaca4ef7b7dd322fd76d81959804705fd8fb633bebb3474fca5686bafaff679306de7ecbc9ccc8d6872a9f943e6e68114de8b3b6b8767111bf98173d9f723b7092127b3e345bee8eb7699e7ed0bf717b9a267c57c47a3772319c912bdacfc78ed59eb02d0e6c83016d1a7a0ec22ff891f78f0f687d76be5974b0b1873c171d16e3255b35000513573fbb95cb717df137ce6526a418bf9a3bd6bc4e6b337dcab9ecb808166d61dc3e77808b8d613ab3583afa42d761843d380d2d8c1a882f8b72b56ae21608edd7bbc3626d33ce2a757f71f5dfcd1cd5797a38d980984bab087b5edb6692618fd04a1e65e72dfe39e6f3977c6d174f9e19e59125026e14974747d5d2ec9015d72e3f1f8da33d86d97946be91bf1010622d6d2078b0ff9dc74b66ca8f90eaf4e81a229f842644b5cb940f3f47de08ce15942cdb25f5db";
const A: &str = "527b274487f59bd8eefbd84492170ca8674b908b14fa16cb80b59b843fe8f4f78621852e6d3af2eb5ad52f223942e7d3de2352e43a022190f17f2f23c02b6b2af09eae23ed2e52074b73eefefa8573fed184dcea95c99a20d01046dbffe5fd17d3a8c7616de49a10b990183111c8b2032386cec4ec5c3e8d70ca7696d3cea2c2a9ab970c22e9706ee34f7ffff1a0d4af09605dc1814fd4ee6dbea2d430b6a25fe47bffbe221a95db81bdbc0a213d263c5e13e10c89e9d3f46155a3f3b944064c819376140a09b61a2f4e62c21665e4d4e2b991f92a23720888e72726456d6b3f79805532cef564307c7d119e05d06572387109d46d88008419ad80a4da83f6fa5a25b1e955b7dd14bb06fbbc54553cd955174c5df33eb1e540f00389a06f0e9a2ceec8fd1c67bc750d3abbdacf39b75194fbb35a36c9a1657539ca7d82dbcb5aa25bf45f8ca78ae894fdbf30feaf6ad4e1a97a57a86f4f0cd46b22b0c4e74ff42a5c65143a2f91043e00ad29c60094526cac43354c4895bbcdcc5a0db12818c3";
const B_PUB: &str = "a0cb1f7e2e598b1a054be6f0ca39e1310821f8bcde1e804c5c87d048c0eaf0bf6f0b28be22c58036926014a32953e498eb5c520cd45e312da421ecd5fd2c652b322d79158e2ffd75392f59add41a505aa6bbe5b9ade7de682411dea91f53cf30605db97560e28b6bbae8f12e65c23d6e50eb6df2ecdef6ba5bdb7f245f26da064862735d5f44dfa2001e441d60b9fb798170e6ccbd07c01157d17b6dc7fdda13db7c17a8a38a7110cd9cdc7d23add91764253dc29b8b6d617ecd335bbd0cd66b18b74fe99d29d70ccd7908efad83aa4656e948edc0165f4f529b49d270acc6b4a20195aed2e4bfec82ab201dd09947b14660ff9e8ace5465fd6a266aa850bfa6bcaa8b52d0fc99f18209a0ae09f073cbdba702e6c00dd72cb24a49943d8e9d008ce310e75e7e8f8c14531b53844efa6eeb74e110149feed3c0bb8d981c386eac7eb389d75d90d624874f83053fdaf3a36f3e31f5ccdf52f4fcfbb398e0200b4d7a40f2c3a65e9e1d834ec4d8821f1903fc6e940e07e82d56757ebfa95742cfe2";
const M1: &str = "9b1dfe9dee8898d139534a4df8dab4a7f7debd436e5cbc73a3b9d1b271711224e829c1abccfce184cf4631fdbbfcc8b6289839462f367f712c3064d128f7ab56";
const M2: &str = "8f3f465ba43be65f24a9c66390d644ac35b3c165bea1c3cfced2fb5edb08a01bf904d62b092f492f40840bd76c00b3fba33886d6ae6b2c8e0f63aba3d7d6f3b7";
const K: &str = "b612e5df51eab4195e55e6bc2733eb18f18152fd9d40db549604aaa2ef0aa9bb53e11c1043d9b84c875be0973ffddc6d9948076273c9a65fff624addcb10745c";

#[test]
fn srp_verifier_matches_hap_nodejs() {
    let v = srp::compute_verifier(&hex(SALT), PASSWORD);
    assert_eq!(v.to_vec(), hex(V));
}

#[test]
fn srp_public_key_matches_hap_nodejs() {
    let server = srp::PairSetupServer::with_secrets(PASSWORD, salt16(), &B_PRIV);
    assert_eq!(server.salt().to_vec(), hex(SALT));
    assert_eq!(server.public_key().to_vec(), hex(B_PUB));
}

#[test]
fn srp_verify_matches_hap_nodejs_proofs_and_key() {
    let server = srp::PairSetupServer::with_secrets(PASSWORD, salt16(), &B_PRIV);
    let outcome = server.verify(&hex(A), &hex(M1)).expect("proof verifies");
    assert_eq!(outcome.server_proof.to_vec(), hex(M2));
    assert_eq!(outcome.session_key.to_vec(), hex(K));
}

#[test]
fn srp_verify_rejects_wrong_proof() {
    let server = srp::PairSetupServer::with_secrets(PASSWORD, salt16(), &B_PRIV);
    let mut bad = hex(M1);
    bad[0] ^= 0xff;
    assert_eq!(server.verify(&hex(A), &bad).err(), Some(HapError::BadProof));
}

#[test]
fn srp_verify_rejects_bad_public_key() {
    let server = srp::PairSetupServer::with_secrets(PASSWORD, salt16(), &B_PRIV);
    // A = 0 (== 0 mod N) must be rejected.
    let zero = [0u8; srp::N_LEN];
    assert_eq!(
        server.verify(&zero, &hex(M1)).err(),
        Some(HapError::InvalidPublicKey)
    );
}

// --- HKDF-SHA-512 (labels from HAP §5.6/§6.5.2) ---

#[test]
fn hkdf_matches_hap_nodejs() {
    let k = hex(K);
    let cases = [
        (
            "Pair-Setup-Encrypt-Salt",
            "Pair-Setup-Encrypt-Info",
            "faf15ed2e543604ba7c9c1af42c0677ef643ec465784590b85253f876a7447b9",
        ),
        (
            "Pair-Setup-Controller-Sign-Salt",
            "Pair-Setup-Controller-Sign-Info",
            "5ba335c229876c093129d2dd774acc937df66284df4ca31e1f78e7fb0f1662b8",
        ),
        (
            "Pair-Setup-Accessory-Sign-Salt",
            "Pair-Setup-Accessory-Sign-Info",
            "c0499e07360deb993e177da830076abbf0e4eb7abb9570f8f0da058f374e332f",
        ),
        (
            "Control-Salt",
            "Control-Write-Encryption-Key",
            "8e6048e4981d7632b70b7f942b6fe56c5ba7aa9bf053dc60ec6a9a7e619fba8f",
        ),
        (
            "Control-Salt",
            "Control-Read-Encryption-Key",
            "f9a43d361fb7f03c389df2a98af25ddf0844e750ec6bcdfca2a173a05d78c3ff",
        ),
    ];
    for (salt, info, expected) in cases {
        let got = hkdf::hkdf_sha512_32(salt.as_bytes(), &k, info.as_bytes()).unwrap();
        assert_eq!(got.to_vec(), hex(expected), "salt={salt} info={info}");
    }
}

#[test]
fn pair_setup_key_helpers_match_hkdf() {
    let k = hex(K);
    assert_eq!(
        pair_setup::encrypt_key(&k).unwrap().to_vec(),
        hex("faf15ed2e543604ba7c9c1af42c0677ef643ec465784590b85253f876a7447b9")
    );
    assert_eq!(
        pair_setup::controller_sign_material(&k).unwrap().to_vec(),
        hex("5ba335c229876c093129d2dd774acc937df66284df4ca31e1f78e7fb0f1662b8")
    );
    assert_eq!(
        pair_setup::accessory_sign_material(&k).unwrap().to_vec(),
        hex("c0499e07360deb993e177da830076abbf0e4eb7abb9570f8f0da058f374e332f")
    );
}

// --- Pair Setup encrypted exchange (ChaCha20-Poly1305, nonce PS-Msg06) ---

#[test]
fn pair_setup_m6_encrypt_matches_hap_nodejs() {
    let k = hex(K);
    let plain = hex("48656c6c6f2c20484150210a");
    let ct = pair_setup::encrypt_m6(&k, &plain).unwrap();
    assert_eq!(
        ct,
        hex("0ab103a38ee69791017822400f6606a2a4fd18f159802f4e0a4662b5")
    );
    // Round-trip: the same key/nonce label pair (PS-Msg06) decrypts it.
    let key = pair_setup::encrypt_key(&k).unwrap();
    let back = chacha::open(&key, &chacha::nonce(b"PS-Msg06"), &[], &ct).unwrap();
    assert_eq!(back, plain);
}

// --- Secure session transport (ChaCha20-Poly1305, counter nonce) ---

#[test]
fn session_frame_matches_hap_nodejs() {
    // AccessoryToController (read) key, first frame (counter 0), aad = LE length.
    let k = hex(K);
    let mut acc = session::HapSession::accessory(&k).unwrap();
    let plain = hex("48656c6c6f2c20484150210a");
    let framed = acc.encrypt(&plain).unwrap();
    // 2-byte length prefix + ciphertext + tag.
    assert_eq!(&framed[0..2], &[plain.len() as u8, 0]);
    assert_eq!(
        &framed[2..],
        hex("d55b5467fd57bfe6112562cb9c84b51cb68089bcdb606f54d084163f").as_slice()
    );
}

#[test]
fn session_round_trip_multi_frame() {
    let k = hex(K);
    let mut acc = session::HapSession::accessory(&k).unwrap();
    let mut ctrl = session::HapSession::controller(&k).unwrap();

    // A payload spanning several frames exercises counter advancement.
    let big: Vec<u8> = (0..(session::MAX_FRAME_LEN * 2 + 7))
        .map(|i| (i % 251) as u8)
        .collect();
    let framed = acc.encrypt(&big).unwrap();
    let recovered = ctrl.decrypt(&framed).unwrap();
    assert_eq!(recovered, big);

    // And the reverse direction, twice, to advance both counters.
    for msg in [b"ping".as_slice(), b"pong".as_slice()] {
        let framed = ctrl.encrypt(msg).unwrap();
        let recovered = acc.decrypt(&framed).unwrap();
        assert_eq!(recovered, msg);
    }
}

#[test]
fn session_rejects_tampered_frame() {
    let k = hex(K);
    let mut acc = session::HapSession::accessory(&k).unwrap();
    let mut ctrl = session::HapSession::controller(&k).unwrap();
    let mut framed = acc.encrypt(b"secret").unwrap();
    let last = framed.len() - 1;
    framed[last] ^= 0x01;
    assert_eq!(ctrl.decrypt(&framed).err(), Some(HapError::Decrypt));
}
