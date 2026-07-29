//! vault — Vault Engine (docs/007-vault.md)
//!
//! Coeur metier : coffre, noeuds, operations utilisateur (create/open/close,
//! import/export, move/rename/delete). Ne chiffre jamais directement, ne
//! touche jamais le disque directement — delegue a crypto/storage/manifest.

use common::NodeId;

/// Type de noeud logique (docs/007-vault.md §4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    File,
    Directory,
}

/// Noeud minimal (squelette).
#[derive(Debug, Clone)]
pub struct Node {
    pub id: NodeId,
    pub kind: NodeKind,
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_is_a_directory() {
        let root = Node { id: NodeId([0u8; 16]), kind: NodeKind::Directory, name: "/".into() };
        assert_eq!(root.kind, NodeKind::Directory);
    }
}
