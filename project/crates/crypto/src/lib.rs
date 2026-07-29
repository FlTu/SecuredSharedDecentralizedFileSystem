//! crypto — primitives approuvees (docs/013-security.md §3)
//!
//! Argon2id (KDF), HKDF-SHA256 (derivation par contexte), XChaCha20-Poly1305
//! (AEAD symetrique), Ed25519 (signature), X25519 (scellement asymetrique).
//! Ne connait jamais les coffres, fichiers ou utilisateurs.
//!
//! TODO (hors squelette) : brancher les vraies bibliotheques (ex. `chacha20poly1305`,
//! `argon2`, `ed25519-dalek`, `x25519-dalek`) une fois la version de Rust cible
//! du projet fixee sur les postes de developpement (rustup, pas la version
//! d'apt utilisee pour ce squelette).

/// Contextes de derivation HKDF distincts (docs/013-security.md §4).
pub const HKDF_CONTEXT_MANIFEST: &str = "syfi-manifest-v1";
pub const HKDF_CONTEXT_BLOCKS: &str = "syfi-blocks-v1";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contexts_are_distinct() {
        assert_ne!(HKDF_CONTEXT_MANIFEST, HKDF_CONTEXT_BLOCKS);
    }
}
