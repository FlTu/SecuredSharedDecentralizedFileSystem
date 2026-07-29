//! ipc — API officiellement supportee de SyFi (docs/012-ipc.md)
//!
//! Protocole custom, CBOR + framing prefixe par longueur, capacites par
//! jeton opaque. Aucune logique metier — route vers les services du daemon
//! apres verification de capacite.

/// Jeton de capacite opaque (docs/012-ipc.md §4.1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityToken(pub [u8; 32]);

/// Portees minimales definies a la compilation (docs/012-ipc.md §4.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    ContactsRead,
    VaultsList,
    SyncStatus,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_is_32_bytes() {
        let t = CapabilityToken([0u8; 32]);
        assert_eq!(t.0.len(), 32);
    }
}
