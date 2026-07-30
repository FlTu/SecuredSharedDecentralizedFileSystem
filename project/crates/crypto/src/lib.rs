//! crypto — primitives approuvees (docs/013-security.md §3)
//!
//! Argon2id (KDF), HKDF-SHA256 (derivation par contexte), XChaCha20-Poly1305
//! (AEAD symetrique), Ed25519 (signature), X25519 (scellement asymetrique,
//! construction "sealed box" - docs/013-security.md §5).
//!
//! Ne connait jamais les coffres, fichiers ou utilisateurs.

use argon2::{Argon2, PasswordHasher, password_hash::{SaltString, rand_core::OsRng as PhcOsRng}};
use chacha20poly1305::{
    aead::{Aead, KeyInit, OsRng},
    XChaCha20Poly1305, XNonce, Key,
};
use ed25519_dalek::{Signer, Verifier, SigningKey, VerifyingKey, Signature};
use hkdf::Hkdf;
use rand_core::RngCore;
use sha2::Sha256;
use x25519_dalek::{StaticSecret, PublicKey as X25519PublicKey};

/// Contextes de derivation HKDF distincts (docs/013-security.md §4).
pub const HKDF_CONTEXT_MANIFEST: &[u8] = b"syfi-manifest-v1";
pub const HKDF_CONTEXT_BLOCKS: &[u8] = b"syfi-blocks-v1";

#[derive(Debug)]
pub struct CryptoError(pub String);

impl std::fmt::Display for CryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CryptoError: {}", self.0)
    }
}
impl std::error::Error for CryptoError {}

// ---------------------------------------------------------------------
// Argon2id : derivation de la cle maitre a partir d'une passphrase
// ---------------------------------------------------------------------

/// Derive une cle maitre de 32 octets a partir d'une passphrase et d'un sel.
/// Le sel doit etre genere une fois a la creation du coffre puis stocke
/// (non secret) a cote du coffre (docs/014-file-format.md).
pub fn derive_master_key(passphrase: &[u8], salt: &SaltString) -> Result<[u8; 32], CryptoError> {
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(passphrase, salt)
        .map_err(|e| CryptoError(format!("argon2id: {e}")))?;
    let hash_bytes = hash.hash.ok_or_else(|| CryptoError("argon2id: pas de hash produit".into()))?;
    let mut key = [0u8; 32];
    let bytes = hash_bytes.as_bytes();
    let n = bytes.len().min(32);
    key[..n].copy_from_slice(&bytes[..n]);
    Ok(key)
}

/// Genere un nouveau sel aleatoire (a stocker en clair a cote du coffre).
pub fn generate_salt() -> SaltString {
    SaltString::generate(&mut PhcOsRng)
}

// ---------------------------------------------------------------------
// HKDF : derivation de sous-cles par contexte (manifest vs blocs)
// ---------------------------------------------------------------------

/// Derive une sous-cle de 32 octets a partir de la cle maitre et d'un contexte
/// (docs/013-security.md §4 : "manifest_key = HKDF(master_key, info = ...)").
pub fn derive_subkey(master_key: &[u8; 32], context: &[u8]) -> [u8; 32] {
    let hk = Hkdf::<Sha256>::new(None, master_key);
    let mut out = [0u8; 32];
    hk.expand(context, &mut out)
        .expect("32 octets est une longueur de sortie valide pour HKDF-SHA256");
    out
}

// ---------------------------------------------------------------------
// XChaCha20-Poly1305 : chiffrement symetrique authentifie
// ---------------------------------------------------------------------

/// Chiffre `plaintext` avec la sous-cle donnee. Retourne (nonce, ciphertext).
/// Le nonce est genere aleatoirement (192 bits — marge tres large, cf.
/// docs/013-security.md §3) et doit etre stocke a cote du ciphertext.
pub fn encrypt(subkey: &[u8; 32], plaintext: &[u8]) -> Result<([u8; 24], Vec<u8>), CryptoError> {
    let cipher = XChaCha20Poly1305::new(Key::from_slice(subkey));
    let mut nonce_bytes = [0u8; 24];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = XNonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| CryptoError(format!("xchacha20poly1305 encrypt: {e}")))?;
    Ok((nonce_bytes, ciphertext))
}

/// Dechiffre un ciphertext produit par `encrypt` avec la meme sous-cle et le
/// meme nonce. Echoue si l'authentification (tag Poly1305) ne correspond pas
/// — donnee corrompue ou mauvaise cle.
pub fn decrypt(subkey: &[u8; 32], nonce: &[u8; 24], ciphertext: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let cipher = XChaCha20Poly1305::new(Key::from_slice(subkey));
    let nonce = XNonce::from_slice(nonce);
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| CryptoError(format!("xchacha20poly1305 decrypt: {e}")))
}

// ---------------------------------------------------------------------
// Ed25519 : signature (utilise pour AccountKey/DeviceKey, docs/011-identity.md)
// ---------------------------------------------------------------------

pub struct Ed25519KeyPair {
    signing_key: SigningKey,
}

impl Ed25519KeyPair {
    pub fn generate() -> Self {
        let mut csprng = rand_core::OsRng;
        Self { signing_key: SigningKey::generate(&mut csprng) }
    }

    pub fn public_key_bytes(&self) -> [u8; 32] {
        self.signing_key.verifying_key().to_bytes()
    }

    pub fn sign(&self, message: &[u8]) -> [u8; 64] {
        self.signing_key.sign(message).to_bytes()
    }
}

/// Verifie une signature Ed25519 (ex. certificat d'appareil signe par la
/// clé de compte, docs/011-identity.md §5).
pub fn verify_signature(public_key: &[u8; 32], message: &[u8], signature: &[u8; 64]) -> bool {
    let Ok(vk) = VerifyingKey::from_bytes(public_key) else { return false };
    let sig = Signature::from_bytes(signature);
    vk.verify(message, &sig).is_ok()
}

// ---------------------------------------------------------------------
// X25519 : scellement asymetrique (enveloppe hybride du manifest,
// docs/013-security.md §5)
// ---------------------------------------------------------------------

pub struct X25519KeyPair {
    secret: StaticSecret,
}

impl X25519KeyPair {
    pub fn generate() -> Self {
        Self { secret: StaticSecret::random_from_rng(OsRng) }
    }

    pub fn public_key_bytes(&self) -> [u8; 32] {
        X25519PublicKey::from(&self.secret).to_bytes()
    }

    /// Calcule un secret partage avec la cle publique d'un pair (Diffie-Hellman).
    fn diffie_hellman(&self, peer_public: &[u8; 32]) -> [u8; 32] {
        let peer = X25519PublicKey::from(*peer_public);
        self.secret.diffie_hellman(&peer).to_bytes()
    }
}

/// Scelle une petite cle de session (32 octets) pour un destinataire donne
/// (docs/013-security.md §5 — "sealed box"). Implementation simplifiee :
/// DH ephemere + HKDF + XChaCha20-Poly1305, dans l'esprit de crypto_box_seal
/// de libsodium (a remplacer par une bibliotheque auditee type `crypto_box`
/// avant tout usage en production — cf. `013-security.md` §8, le format doit
/// rester documente et reproductible independamment de ce code).
pub fn seal_for_recipient(recipient_public: &[u8; 32], session_key: &[u8; 32]) -> Result<Vec<u8>, CryptoError> {
    let ephemeral = X25519KeyPair::generate();
    let shared = ephemeral.diffie_hellman(recipient_public);
    let wrapping_key = derive_subkey(&shared, b"syfi-sealed-box-v1");
    let (nonce, ciphertext) = encrypt(&wrapping_key, session_key)?;

    let mut out = Vec::with_capacity(32 + 24 + ciphertext.len());
    out.extend_from_slice(&ephemeral.public_key_bytes()); // en-tete : cle ephemere publique
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

/// Ouvre une enveloppe scellee avec `seal_for_recipient`, avec la cle privee
/// X25519 du destinataire.
pub fn unseal(recipient: &X25519KeyPair, sealed: &[u8]) -> Result<[u8; 32], CryptoError> {
    if sealed.len() < 32 + 24 {
        return Err(CryptoError("enveloppe scellee trop courte".into()));
    }
    let ephemeral_public: [u8; 32] = sealed[0..32].try_into().unwrap();
    let nonce: [u8; 24] = sealed[32..56].try_into().unwrap();
    let ciphertext = &sealed[56..];

    let shared = recipient.diffie_hellman(&ephemeral_public);
    let wrapping_key = derive_subkey(&shared, b"syfi-sealed-box-v1");
    let plaintext = decrypt(&wrapping_key, &nonce, ciphertext)?;
    plaintext
        .try_into()
        .map_err(|_| CryptoError("cle de session decapsulee de taille inattendue".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contexts_are_distinct() {
        assert_ne!(HKDF_CONTEXT_MANIFEST, HKDF_CONTEXT_BLOCKS);
    }

    #[test]
    fn master_key_derivation_is_deterministic_for_same_salt() {
        let salt = generate_salt();
        let k1 = derive_master_key(b"une passphrase de test", &salt).unwrap();
        let k2 = derive_master_key(b"une passphrase de test", &salt).unwrap();
        assert_eq!(k1, k2);
    }

    #[test]
    fn subkeys_for_manifest_and_blocks_differ() {
        let master = [42u8; 32];
        let manifest_key = derive_subkey(&master, HKDF_CONTEXT_MANIFEST);
        let blocks_key = derive_subkey(&master, HKDF_CONTEXT_BLOCKS);
        assert_ne!(manifest_key, blocks_key);
    }

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let key = [7u8; 32];
        let plaintext = b"contenu d'un bloc de test";
        let (nonce, ciphertext) = encrypt(&key, plaintext).unwrap();
        let decrypted = decrypt(&key, &nonce, &ciphertext).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn decrypt_fails_with_wrong_key() {
        let key = [7u8; 32];
        let wrong_key = [8u8; 32];
        let (nonce, ciphertext) = encrypt(&key, b"secret").unwrap();
        assert!(decrypt(&wrong_key, &nonce, &ciphertext).is_err());
    }

    #[test]
    fn two_encryptions_of_same_content_produce_different_ciphertexts() {
        // Confirme l'absence de dedup par construction (docs/006-storage.md §2) :
        // deux chiffrements du meme contenu en clair, avec des nonces aleatoires
        // distincts, ne produisent jamais le meme ciphertext.
        let key = [1u8; 32];
        let (_, c1) = encrypt(&key, b"meme contenu").unwrap();
        let (_, c2) = encrypt(&key, b"meme contenu").unwrap();
        assert_ne!(c1, c2);
    }

    #[test]
    fn ed25519_sign_and_verify() {
        let kp = Ed25519KeyPair::generate();
        let msg = b"certificat d'appareil";
        let sig = kp.sign(msg);
        assert!(verify_signature(&kp.public_key_bytes(), msg, &sig));
    }

    #[test]
    fn ed25519_verify_fails_for_tampered_message() {
        let kp = Ed25519KeyPair::generate();
        let sig = kp.sign(b"message original");
        assert!(!verify_signature(&kp.public_key_bytes(), b"message modifie", &sig));
    }

    #[test]
    fn sealed_box_roundtrip() {
        let recipient = X25519KeyPair::generate();
        let session_key = [99u8; 32];
        let sealed = seal_for_recipient(&recipient.public_key_bytes(), &session_key).unwrap();
        let opened = unseal(&recipient, &sealed).unwrap();
        assert_eq!(opened, session_key);
    }

    #[test]
    fn sealed_box_fails_for_wrong_recipient() {
        let recipient = X25519KeyPair::generate();
        let attacker = X25519KeyPair::generate();
        let sealed = seal_for_recipient(&recipient.public_key_bytes(), &[1u8; 32]).unwrap();
        assert!(unseal(&attacker, &sealed).is_err());
    }
}
