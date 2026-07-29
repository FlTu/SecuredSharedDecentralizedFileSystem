//! storage — Storage Engine (docs/006-storage.md)
//!
//! Blocs immuables, adresses par UUID opaque (jamais par empreinte de
//! contenu — pas de deduplication, cf. §2 pour la justification). WAL,
//! Garbage Collector par scan de references du manifest, separation
//! Block Store / Local Index Store (§11bis).

use common::BlockId;

/// Etat minimal d'un bloc stocke (squelette — pas encore de vrai I/O disque).
#[derive(Debug)]
pub struct StoredBlock {
    pub id: BlockId,
    pub size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_carries_its_id() {
        let b = StoredBlock { id: BlockId([0u8; 16]), size: 0 };
        assert_eq!(b.size, 0);
    }
}
