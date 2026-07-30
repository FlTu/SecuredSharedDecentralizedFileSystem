//! vault — Vault Engine (docs/007-vault.md)
//!
//! Coeur metier : cree/ouvre/ferme un coffre, importe/exporte des fichiers,
//! liste une arborescence. Delegue le chiffrement a `crypto`, le stockage
//! physique a `storage`, l'indexation a `manifest`. Ne chiffre jamais
//! directement, n'ecrit jamais directement sur le disque en dehors de son
//! propre repertoire de configuration (docs/007-vault.md §25, invariants).

use common::NodeId;
use manifest::{BlockRef, EntryType, IndexEntry, Manifest};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use storage::{BlockStore, BlockToStore};
use uuid::Uuid;

/// Taille de bloc fixe par defaut (docs/006-storage.md §7 — decision : taille
/// fixe, pas de content-defined chunking en V1). 1 Mio pour ce squelette ;
/// a rendre configurable par coffre par la suite.
pub const DEFAULT_CHUNK_SIZE: usize = 1024 * 1024;

/// Identifiant sentinelle pour le parent des noeuds a la racine du coffre.
/// N'existe pas comme entree de manifest — racine implicite
/// (docs/007-vault.md §25 : "le repertoire racine ne peut etre supprime").
pub const ROOT_NODE_ID: NodeId = NodeId([0u8; 16]);

const MANIFEST_FILE: &str = "manifest/index.cbor.enc";
const SALT_FILE: &str = "config/salt.txt";

#[derive(Debug)]
pub enum VaultError {
    Io(io::Error),
    Crypto(String),
    Manifest(String),
    NodeNotFound(NodeId),
    AlreadyExists(PathBuf),
}

impl std::fmt::Display for VaultError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VaultError::Io(e) => write!(f, "erreur I/O: {e}"),
            VaultError::Crypto(s) => write!(f, "erreur cryptographique: {s}"),
            VaultError::Manifest(s) => write!(f, "erreur de manifest: {s}"),
            VaultError::NodeNotFound(id) => write!(f, "noeud introuvable: {:?}", id.0),
            VaultError::AlreadyExists(p) => write!(f, "un coffre existe deja: {}", p.display()),
        }
    }
}
impl std::error::Error for VaultError {}
impl From<io::Error> for VaultError { fn from(e: io::Error) -> Self { VaultError::Io(e) } }
impl From<storage::StorageError> for VaultError {
    fn from(e: storage::StorageError) -> Self { VaultError::Crypto(e.to_string()) }
}

/// Un coffre ouvert (docs/007-vault.md). Tant que cette structure vit, le
/// coffre est `Unlocked` (docs/003-architecture.md Partie 3 §24) ; la
/// fermeture correspond simplement au `Drop` pour ce squelette (le vrai
/// nettoyage memoire des cles — zeroization — sera ajoute avec `zeroize`
/// sur une machine avec un Rust plus recent, cf. README du squelette).
pub struct Vault {
    root: PathBuf,
    blocks: BlockStore,
    manifest: Manifest,
    manifest_key: [u8; 32],
    blocks_key: [u8; 32],
    chunk_size: usize,
}

impl Vault {
    /// Cree un nouveau coffre a l'emplacement donne (docs/007-vault.md §7 :
    /// generation de la cle maitre, initialisation du Manifest, creation de
    /// la structure racine).
    pub fn create(root: impl AsRef<Path>, passphrase: &[u8]) -> Result<Self, VaultError> {
        let root = root.as_ref().to_path_buf();
        if root.exists() && fs::read_dir(&root)?.next().is_some() {
            return Err(VaultError::AlreadyExists(root));
        }
        fs::create_dir_all(root.join("blocks"))?;
        fs::create_dir_all(root.join("manifest"))?;
        fs::create_dir_all(root.join("wal"))?;
        fs::create_dir_all(root.join("cache"))?;
        fs::create_dir_all(root.join("tmp"))?;
        fs::create_dir_all(root.join("config"))?;

        let salt = crypto::generate_salt();
        fs::write(root.join(SALT_FILE), salt.as_str())?;

        let master_key = crypto::derive_master_key(passphrase, &salt)
            .map_err(|e| VaultError::Crypto(e.to_string()))?;
        let manifest_key = crypto::derive_subkey(&master_key, crypto::HKDF_CONTEXT_MANIFEST);
        let blocks_key = crypto::derive_subkey(&master_key, crypto::HKDF_CONTEXT_BLOCKS);

        let blocks = BlockStore::open(root.join("blocks"))?;
        let manifest = Manifest::new();

        let mut vault = Self { root, blocks, manifest, manifest_key, blocks_key, chunk_size: DEFAULT_CHUNK_SIZE };
        vault.persist_manifest()?;
        Ok(vault)
    }

    /// Ouvre un coffre existant (docs/007-vault.md §8 : verifie le format,
    /// deverrouille les cles, charge le Manifest).
    pub fn open(root: impl AsRef<Path>, passphrase: &[u8]) -> Result<Self, VaultError> {
        let root = root.as_ref().to_path_buf();
        let salt_str = fs::read_to_string(root.join(SALT_FILE))?;
        let salt = crypto::generate_salt(); // place-holder pour le type ; remplace ci-dessous
        let salt = argon2_salt_from_str(&salt_str).unwrap_or(salt);

        let master_key = crypto::derive_master_key(passphrase, &salt)
            .map_err(|e| VaultError::Crypto(e.to_string()))?;
        let manifest_key = crypto::derive_subkey(&master_key, crypto::HKDF_CONTEXT_MANIFEST);
        let blocks_key = crypto::derive_subkey(&master_key, crypto::HKDF_CONTEXT_BLOCKS);

        let sealed = fs::read(root.join(MANIFEST_FILE))?;
        if sealed.len() < 24 {
            return Err(VaultError::Manifest("fichier de manifest tronque".into()));
        }
        let (nonce_bytes, ciphertext) = sealed.split_at(24);
        let nonce: [u8; 24] = nonce_bytes.try_into().unwrap();
        let manifest = Manifest::unseal(&manifest_key, &nonce, ciphertext)
            .map_err(|e| VaultError::Manifest(e.to_string()))?;

        let blocks = BlockStore::open(root.join("blocks"))?;
        Ok(Self { root, blocks, manifest, manifest_key, blocks_key, chunk_size: DEFAULT_CHUNK_SIZE })
    }

    fn persist_manifest(&mut self) -> Result<(), VaultError> {
        let (nonce, ciphertext) = self.manifest.seal(&self.manifest_key)
            .map_err(|e| VaultError::Manifest(e.to_string()))?;
        let mut out = Vec::with_capacity(24 + ciphertext.len());
        out.extend_from_slice(&nonce);
        out.extend_from_slice(&ciphertext);
        // Ecriture atomique (tmp + rename), meme principe que pour les blocs
        // (docs/006-storage.md §8).
        let tmp = self.root.join("tmp/manifest.tmp");
        fs::write(&tmp, &out)?;
        fs::rename(&tmp, self.root.join(MANIFEST_FILE))?;
        Ok(())
    }

    /// Importe un fichier local dans le coffre, sous le noeud parent donne
    /// (`ROOT_NODE_ID` pour la racine). Pipeline complet (docs/007-vault.md
    /// §10) : lecture -> chunking taille fixe -> chiffrement -> stockage ->
    /// mise a jour du manifest -> commit (persistance du manifest scelle).
    pub fn import_file(&mut self, source: impl AsRef<Path>, parent: NodeId, name: &str) -> Result<NodeId, VaultError> {
        let data = fs::read(source)?;
        let chunks = storage::chunk_fixed_size(&data, self.chunk_size);

        let mut block_refs = Vec::with_capacity(chunks.len());
        for chunk in &chunks {
            let (nonce, ciphertext) = crypto::encrypt(&self.blocks_key, chunk)
                .map_err(|e| VaultError::Crypto(e.to_string()))?;
            let block_id = self.blocks.write_block(BlockToStore { nonce, ciphertext: &ciphertext })?;
            block_refs.push(BlockRef { block_id: *block_id.as_bytes(), nonce });
        }

        let node_id = *Uuid::new_v4().as_bytes();
        let entry = IndexEntry {
            node_id,
            parent_id: parent.0,
            name: name.to_string(),
            entry_type: EntryType::File,
            size: data.len() as u64,
            blocks: block_refs,
            deleted: false,
            version_vector: HashMap::new(),
        };
        self.manifest.insert(entry);
        self.persist_manifest()?;
        Ok(NodeId(node_id))
    }

    /// Cree un dossier logique (aucun bloc associe — docs/007-vault.md §9).
    pub fn create_directory(&mut self, parent: NodeId, name: &str) -> Result<NodeId, VaultError> {
        let node_id = *Uuid::new_v4().as_bytes();
        let entry = IndexEntry {
            node_id,
            parent_id: parent.0,
            name: name.to_string(),
            entry_type: EntryType::Directory,
            size: 0,
            blocks: vec![],
            deleted: false,
            version_vector: HashMap::new(),
        };
        self.manifest.insert(entry);
        self.persist_manifest()?;
        Ok(NodeId(node_id))
    }

    /// Exporte (dechiffre et reassemble) un fichier du coffre vers un chemin
    /// local (docs/007-vault.md §11).
    pub fn export_file(&self, node: NodeId, dest: impl AsRef<Path>) -> Result<(), VaultError> {
        let entry = self.manifest.get(&node).ok_or(VaultError::NodeNotFound(node))?;
        let mut out = Vec::with_capacity(entry.size as usize);
        for block_ref in &entry.blocks {
            let id = Uuid::from_bytes(block_ref.block_id);
            let (nonce, ciphertext) = self.blocks.read_block(&id)
                .map_err(|e| VaultError::Crypto(e.to_string()))?;
            let plaintext = crypto::decrypt(&self.blocks_key, &nonce, &ciphertext)
                .map_err(|e| VaultError::Crypto(e.to_string()))?;
            out.extend_from_slice(&plaintext);
        }
        fs::write(dest, out)?;
        Ok(())
    }

    /// Liste le contenu (non supprime) d'un dossier — utilise par
    /// l'explorateur virtuel avant tout montage FUSE reel
    /// (docs/015-roadmap.md Phase 9).
    pub fn list_directory(&self, parent: NodeId) -> Vec<&IndexEntry> {
        self.manifest.list_children(&parent)
    }

    pub fn delete_node(&mut self, node: NodeId) -> Result<(), VaultError> {
        self.manifest.mark_deleted(&node).map_err(|e| VaultError::Manifest(e.to_string()))?;
        self.persist_manifest()?;
        Ok(())
    }

    pub fn node_count(&self) -> usize {
        self.manifest.len()
    }
}

/// Reconstruit un `SaltString` a partir de sa forme texte stockee
/// (docs/006-storage.md §5 : le sel n'est pas secret, stocke en clair a
/// cote du coffre).
fn argon2_salt_from_str(s: &str) -> Option<argon2::password_hash::SaltString> {
    argon2::password_hash::SaltString::from_b64(s.trim()).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn temp_vault_dir(name: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("syfi-vault-test-{name}-{}", Uuid::new_v4()));
        p
    }

    #[test]
    fn create_then_open_roundtrip() {
        let dir = temp_vault_dir("create-open");
        {
            let _v = Vault::create(&dir, b"passphrase de test").unwrap();
        }
        let v = Vault::open(&dir, b"passphrase de test").unwrap();
        assert_eq!(v.node_count(), 0);
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn open_with_wrong_passphrase_produces_different_keys_and_fails_to_unseal() {
        let dir = temp_vault_dir("wrong-pass");
        {
            let _v = Vault::create(&dir, b"bon mot de passe").unwrap();
        }
        let result = Vault::open(&dir, b"mauvais mot de passe");
        assert!(result.is_err());
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn import_then_export_roundtrip() {
        let dir = temp_vault_dir("import-export");
        let mut v = Vault::create(&dir, b"passphrase").unwrap();

        let src = dir.join("source.txt");
        fs::File::create(&src).unwrap().write_all(b"contenu du fichier de test, plusieurs octets ici").unwrap();

        let node = v.import_file(&src, ROOT_NODE_ID, "source.txt").unwrap();

        let dest = dir.join("export.txt");
        v.export_file(node, &dest).unwrap();

        let exported = fs::read(&dest).unwrap();
        assert_eq!(exported, b"contenu du fichier de test, plusieurs octets ici");
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn import_survives_vault_close_and_reopen() {
        let dir = temp_vault_dir("persist");
        let src = dir.join("a.txt"); // cree apres, mais le chemin ne depend pas de l'ouverture du coffre

        let node = {
            let mut v = Vault::create(&dir, b"passphrase").unwrap();
            fs::write(&src, b"donnees persistantes").unwrap();
            v.import_file(&src, ROOT_NODE_ID, "a.txt").unwrap()
        }; // le coffre est "ferme" (Drop) ici

        let v2 = Vault::open(&dir, b"passphrase").unwrap();
        let children = v2.list_directory(ROOT_NODE_ID);
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].name, "a.txt");

        let dest = dir.join("out.txt");
        v2.export_file(node, &dest).unwrap();
        assert_eq!(fs::read(dest).unwrap(), b"donnees persistantes");
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn deleted_node_is_a_tombstone_not_removed() {
        let dir = temp_vault_dir("tombstone");
        let mut v = Vault::create(&dir, b"passphrase").unwrap();
        let src = dir.join("b.txt");
        fs::write(&src, b"a supprimer").unwrap();
        let node = v.import_file(&src, ROOT_NODE_ID, "b.txt").unwrap();

        v.delete_node(node).unwrap();

        assert_eq!(v.list_directory(ROOT_NODE_ID).len(), 0); // filtre les tombstones
        assert_eq!(v.node_count(), 1); // l'entree existe toujours (tombstone)
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn large_file_is_split_into_multiple_blocks() {
        let dir = temp_vault_dir("multiblock");
        let mut v = Vault::create(&dir, b"passphrase").unwrap();
        v.chunk_size = 16; // force plusieurs blocs pour un petit fichier de test

        let src = dir.join("big.bin");
        fs::write(&src, vec![7u8; 100]).unwrap();
        let node = v.import_file(&src, ROOT_NODE_ID, "big.bin").unwrap();

        let entry = v.manifest.get(&node).unwrap();
        assert!(entry.blocks.len() > 1);

        let dest = dir.join("big_out.bin");
        v.export_file(node, &dest).unwrap();
        assert_eq!(fs::read(dest).unwrap(), vec![7u8; 100]);
        fs::remove_dir_all(dir).ok();
    }
}
