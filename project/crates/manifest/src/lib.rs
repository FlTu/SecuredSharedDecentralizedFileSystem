//! manifest — Manifest Engine (docs/008-manifest.md, docs/014-file-format.md §4)
//!
//! Index logique partitionne : Header, Partitions, Journal, Merkle Index,
//! Version Vectors. Ne connait jamais les cles ni le disque directement.

use common::{BlockId, NodeId};
use std::collections::HashMap;

/// Reference vers un bloc chiffre, telle que stockee dans une entree d'index
/// (docs/014-file-format.md §4.1).
#[derive(Debug, Clone)]
pub struct BlockRef {
    pub block_id: BlockId,
    pub nonce: [u8; 24],
}

/// Entree d'index minimale (squelette — vecteur de version simplifie en Map<DeviceId, u64>
/// une fois la crate `identity` disponible ; ici place-holder avec un identifiant brut).
#[derive(Debug, Clone)]
pub struct IndexEntry {
    pub node_id: NodeId,
    pub parent_id: NodeId,
    pub blocks: Vec<BlockRef>,
    pub deleted: bool,
    pub version_vector: HashMap<[u8; 16], u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tombstone_flag_defaults_off() {
        let e = IndexEntry {
            node_id: NodeId([0u8; 16]),
            parent_id: NodeId([0u8; 16]),
            blocks: vec![],
            deleted: false,
            version_vector: HashMap::new(),
        };
        assert!(!e.deleted);
    }
}
