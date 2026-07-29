//! common — types partages, erreurs, evenements, identifiants (docs/004-workspace.md §7)
//!
//! Ne contient aucune logique metier. Toutes les autres crates du Core
//! peuvent en dependre ; elle ne depend d'aucune d'entre elles.

/// Identifiant de noeud (cf. docs/007-vault.md §5 — UUID v7 dans l'implementation finale).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub [u8; 16]);

/// Identifiant de bloc (cf. docs/006-storage.md §2 — UUID v4 opaque, jamais derive du contenu).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(pub [u8; 16]);

/// Erreur racine commune. Chaque moteur definira sa propre variante enrichie
/// (StorageError, VaultError, SyncError, ...) — cf. docs/003-architecture.md
/// Partie 5 §66.
#[derive(Debug)]
pub struct SyFiError {
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_distinct_types() {
        let n = NodeId([1u8; 16]);
        let b = BlockId([1u8; 16]);
        assert_eq!(n.0, b.0); // meme octets, types differents (pas de confusion possible)
    }
}
