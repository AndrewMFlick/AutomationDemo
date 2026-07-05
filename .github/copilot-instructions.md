# hap-rs - Rust port of HAP-NodeJS

## Reference vs target
- `reference/HAP-NodeJS/` = source of truth [TypeScript]. NEVER modify.
- `crates/**`             = target [Rust].
- Spec: HAP R2 [Apple HomeKit Accessory Protocol, Non-Commercial].

## Model routing - respect issue labels
- `model:foundry-local` -> Runs on Foundry Local. Mechanical work.
- `model:haiku`   -> Fast cloud model for summarization.
- `model:codex`   -> TLV8 and well-patterned wire code.
- `model:deepseek`-> Boilerplate struct/enum translation.
- `model:opus`    -> SRP, ChaCha20-Poly1305, X25519/Ed25519. Byte-exact.
- `model:fine-tuned` -> Adversarial spec review, air-gapped, Foundry Local.

## Rust rules
- Edition 2021, MSRV 1.75.
- `#![forbid(unsafe_code)]` everywhere unless FFI-justified.
- Crypto crates: ring, chacha20poly1305, x25519-dalek, ed25519-dalek, srp.
- No `unwrap()` in library code. Return `HapError`.
- Every PR: cargo fmt --check + cargo clippy -D warnings + cargo nextest + cargo audit.

## Compliance
- Changes to `hap-crypto` or `hap-http` need `security-review-required`.
- PR body must cite HAP spec section touched.
