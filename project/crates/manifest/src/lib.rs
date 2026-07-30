//! manifest — Manifest Engine (docs/008-manifest.md, docs/014-file-format.md §4)
//!
//! Index logique en memoire, serialise en CBOR, chiffre au repos avec la
//! sous-cle "manifest" (contexte HKDF distinct des blocs, cf.
//! docs/013-security.md §4). Ne connait jamais le reseau ni les blocs
//! chiffres eux-memes (seulement leurs identifiants).

use common::{BlockId, NodeId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io;

#[derive(Debug)]
pub enum ManifestError {
    Io(io::Error),
    Serialization(String),
    Crypto(String),
    UnknownNode(NodeId),
}

impl std::fmt::Display for ManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ManifestError::Io(e) => write!(f, "erreur I/O: {e}"),
            ManifestError::Serialization(s) => write!(f, "erreur de serialisation: {s}"),
            ManifestError::Crypto(s) => write!(f, "erreur cryptographique: {s}"),
            ManifestError::UnknownNode(id) => write!(f, "noeud inconnu: {:?}", id.0),
        }
    }
}
impl std::error::Error for ManifestError {}
impl From<io::Error> for ManifestError {
    fn from(e: io::Error) -> Self { ManifestError::Io(e) }
}

/// Reference vers un bloc chiffre (docs/014-file-format.md §4.1).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockRef {
    pub block_id: [u8; 16],
    pub nonce: [u8; 24],
}

impl BlockRef {
    pub fn block_id(&self) -> BlockId {
        BlockId(self.block_id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntryType {
    File,
    Directory,
}

/// Entree d'index (docs/008-manifest.md §8-9, docs/014-file-format.md §4.1).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexEntry {
    pub node_id: [u8; 16],
    pub parent_id: [u8; 16],
    pub name: String,
    pub entry_type: EntryType,
    pub size: u64,
    pub blocks: Vec<BlockRef>,
    pub deleted: bool,
    /// Vecteur de version simplifie : device_id -> compteur (docs/009-sync.md §6).
    pub version_vector: HashMap<[u8; 16], u64>,
}

/// Index complet d'un coffre, garde en memoire pendant que le coffre est
/// ouvert (docs/008-manifest.md §3 : "immuable, partitionne, deterministe,
/// serialisable, reconstruisible" — le partitionnement reel viendra avec la
/// montee en charge, ce squelette garde une seule table pour le MVP local).
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Manifest {
    entries: HashMap<[u8; 16], IndexEntry>,
}

impl Manifest {
    pub fn new() -> Self {
        Self { entries: HashMap::new() }
    }

    pub fn insert(&mut self, entry: IndexEntry) {
        self.entries.insert(entry.node_id, entry);
    }

    pub fn get(&self, id: &NodeId) -> Option<&IndexEntry> {
        self.entries.get(&id.0)
    }

    /// Liste les enfants directs (non supprimes) d'un noeud parent —
    /// utilise notamment par l'explorateur virtuel (docs/010-network.md,
    /// futur explorateur avant tout montage FUSE reel, docs/015-roadmap.md
    /// Phase 9).
    pub fn list_children(&self, parent: &NodeId) -> Vec<&IndexEntry> {
        self.entries
            .values()
            .filter(|e| e.parent_id == parent.0 && !e.deleted)
            .collect()
    }

    /// Marque une entree comme supprimee (tombstone, docs/009-sync.md §9) —
    /// ne retire jamais l'entree elle-meme.
    pub fn mark_deleted(&mut self, id: &NodeId) -> Result<(), ManifestError> {
        let entry = self.entries.get_mut(&id.0).ok_or(ManifestError::UnknownNode(*id))?;
        entry.deleted = true;
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Serialise l'index en CBOR (docs/014-file-format.md §4).
    fn to_cbor(&self) -> Result<Vec<u8>, ManifestError> {
        let mut buf = Vec::new();
        ciborium::into_writer(self, &mut buf)
            .map_err(|e| ManifestError::Serialization(e.to_string()))?;
        Ok(buf)
    }

    fn from_cbor(bytes: &[u8]) -> Result<Self, ManifestError> {
        ciborium::from_reader(bytes).map_err(|e| ManifestError::Serialization(e.to_string()))
    }

    /// Chiffre l'index avec la sous-cle "manifest" et retourne (nonce, ciphertext),
    /// prets a etre ecrits sur disque (docs/013-security.md §4).
    pub fn seal(&self, manifest_key: &[u8; 32]) -> Result<([u8; 24], Vec<u8>), ManifestError> {
        let plaintext = self.to_cbor()?;
        crypto::encrypt(manifest_key, &plaintext).map_err(|e| ManifestError::Crypto(e.to_string()))
    }

    /// Dechiffre un manifest precedemment scelle par `seal`.
    pub fn unseal(manifest_key: &[u8; 32], nonce: &[u8; 24], ciphertext: &[u8]) -> Result<Self, ManifestError> {
        let plaintext = crypto::decrypt(manifest_key, nonce, ciphertext)
            .map_err(|e| ManifestError::Crypto(e.to_string()))?;
        Self::from_cbor(&plaintext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_entry(id: [u8; 16], parent: [u8; 16], name: &str, kind: EntryType) -> IndexEntry {
        IndexEntry {
            node_id: id,
            parent_id: parent,
            name: name.to_string(),
            entry_type: kind,
            size: 0,
            blocks: vec![],
            deleted: false,
            version_vector: HashMap::new(),
        }
    }

    #[test]
    fn list_children_excludes_tombstones() {
        let mut m = Manifest::new();
        let root = [0u8; 16];
        m.insert(sample_entry([1u8; 16], root, "a.txt", EntryType::File));
        m.insert(sample_entry([2u8; 16], root, "b.txt", EntryType::File));
        m.mark_deleted(&NodeId([2u8; 16])).unwrap();

        let children = m.list_children(&NodeId(root));
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].name, "a.txt");
    }

    #[test]
    fn seal_unseal_roundtrip() {
        let mut m = Manifest::new();
        m.insert(sample_entry([1u8; 16], [0u8; 16], "doc.txt", EntryType::File));
        let key = [5u8; 32];

        let (nonce, ciphertext) = m.seal(&key).unwrap();
        let reloaded = Manifest::unseal(&key, &nonce, &ciphertext).unwrap();

        assert_eq!(reloaded.len(), 1);
        assert_eq!(reloaded.get(&NodeId([1u8; 16])).unwrap().name, "doc.txt");
    }

    #[test]
    fn unseal_fails_with_wrong_key() {
        let mut m = Manifest::new();
        m.insert(sample_entry([1u8; 16], [0u8; 16], "doc.txt", EntryType::File));
        let key = [5u8; 32];
        let wrong_key = [6u8; 32];

        let (nonce, ciphertext) = m.seal(&key).unwrap();
        assert!(Manifest::unseal(&wrong_key, &nonce, &ciphertext).is_err());
    }

    #[test]
    fn mark_deleted_on_unknown_node_fails() {
        let mut m = Manifest::new();
        assert!(m.mark_deleted(&NodeId([9u8; 16])).is_err());
    }
}
