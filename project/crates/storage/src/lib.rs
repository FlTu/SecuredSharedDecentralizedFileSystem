//! storage — Storage Engine (docs/006-storage.md)
//!
//! Blocs immuables, adresses par UUID opaque (jamais par empreinte de
//! contenu — pas de deduplication, cf. §2 pour la justification). WAL
//! minimal, Garbage Collector par scan de references du manifest (a
//! implementer cote `manifest`/`vault`), separation Block Store / Local
//! Index Store (§11bis) : ce module ne gere que le Block Store.

use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug)]
pub enum StorageError {
    Io(io::Error),
    CorruptedHeader(String),
    InvalidChecksum,
    BlockNotFound(Uuid),
}

impl From<io::Error> for StorageError {
    fn from(e: io::Error) -> Self {
        StorageError::Io(e)
    }
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::Io(e) => write!(f, "erreur I/O: {e}"),
            StorageError::CorruptedHeader(s) => write!(f, "en-tete corrompu: {s}"),
            StorageError::InvalidChecksum => write!(f, "checksum invalide"),
            StorageError::BlockNotFound(id) => write!(f, "bloc introuvable: {id}"),
        }
    }
}
impl std::error::Error for StorageError {}

const MAGIC: &[u8; 4] = b"SYFB";
const FORMAT_VERSION: u16 = 1;

/// Un bloc pret a etre stocke : ciphertext + nonce, tel que produit par
/// `crypto::encrypt` (docs/014-file-format.md §3).
pub struct BlockToStore<'a> {
    pub nonce: [u8; 24],
    pub ciphertext: &'a [u8],
}

/// Espace de stockage physique des blocs (Block Store, docs/006-storage.md §11bis).
/// Ne connait ni les cles ni la structure logique — uniquement des blocs
/// opaques identifies par UUID.
pub struct BlockStore {
    root: PathBuf,
}

impl BlockStore {
    /// Ouvre (ou cree) un Block Store a l'emplacement donne
    /// (docs/006-storage.md §5 : "vault/blocks/").
    pub fn open(root: impl AsRef<Path>) -> Result<Self, StorageError> {
        let root = root.as_ref().to_path_buf();
        fs::create_dir_all(&root)?;
        Ok(Self { root })
    }

    fn path_for(&self, id: &Uuid) -> PathBuf {
        self.root.join(format!("{id}.blk"))
    }

    /// Ecrit un nouveau bloc. Genere un `BlockId` a l'ecriture (jamais
    /// derive du contenu, cf. docs/006-storage.md §2) et retourne cet id.
    ///
    /// Format sur disque (docs/014-file-format.md §3) :
    /// magic(4) | format_version(2) | block_id(16) | payload_size(4)
    /// | nonce(24) | ciphertext | integrity_hash(32, BLAKE3 — ici un
    /// espace reserve, la vraie fonction de hash sera branchee cote
    /// crypto une fois BLAKE3 ajoute au workspace).
    pub fn write_block(&self, block: BlockToStore<'_>) -> Result<Uuid, StorageError> {
        let id = Uuid::new_v4();
        let path = self.path_for(&id);

        // Ecriture dans un fichier temporaire puis rename atomique — evite
        // qu'un bloc partiellement ecrit soit jamais considere comme valide
        // (docs/006-storage.md §8 ; le WAL applicatif complet, avec
        // TransactionID et rollback, sera ajoute avec le Vault Engine qui
        // orchestre des operations multi-blocs).
        let tmp_path = self.root.join(format!("{id}.tmp"));
        {
            let mut f = fs::File::create(&tmp_path)?;
            f.write_all(MAGIC)?;
            f.write_all(&FORMAT_VERSION.to_le_bytes())?;
            f.write_all(id.as_bytes())?;
            f.write_all(&(block.ciphertext.len() as u32).to_le_bytes())?;
            f.write_all(&block.nonce)?;
            f.write_all(block.ciphertext)?;
            f.sync_all()?;
        }
        fs::rename(&tmp_path, &path)?;
        Ok(id)
    }

    /// Lit un bloc par son identifiant. Verifie le magic, la version de
    /// format et la coherence de taille avant de retourner le nonce et le
    /// ciphertext (docs/006-storage.md §9-10).
    pub fn read_block(&self, id: &Uuid) -> Result<([u8; 24], Vec<u8>), StorageError> {
        let path = self.path_for(id);
        let mut f = fs::File::open(&path).map_err(|e| {
            if e.kind() == io::ErrorKind::NotFound {
                StorageError::Io(e) // remonte, le CLI/manifest traduira en BlockNotFound au besoin
            } else {
                StorageError::Io(e)
            }
        })?;

        let mut magic = [0u8; 4];
        f.read_exact(&mut magic)?;
        if &magic != MAGIC {
            return Err(StorageError::CorruptedHeader("magic invalide".into()));
        }

        let mut version_bytes = [0u8; 2];
        f.read_exact(&mut version_bytes)?;
        let version = u16::from_le_bytes(version_bytes);
        if version != FORMAT_VERSION {
            return Err(StorageError::CorruptedHeader(format!(
                "version de format non supportee: {version}"
            )));
        }

        let mut stored_id_bytes = [0u8; 16];
        f.read_exact(&mut stored_id_bytes)?;
        if stored_id_bytes != *id.as_bytes() {
            return Err(StorageError::CorruptedHeader(
                "l'identifiant stocke ne correspond pas au nom de fichier".into(),
            ));
        }

        let mut size_bytes = [0u8; 4];
        f.read_exact(&mut size_bytes)?;
        let size = u32::from_le_bytes(size_bytes) as usize;

        let mut nonce = [0u8; 24];
        f.read_exact(&mut nonce)?;

        let mut ciphertext = vec![0u8; size];
        f.read_exact(&mut ciphertext)?;

        Ok((nonce, ciphertext))
    }

    /// Supprime un bloc. Ne verifie aucune reference elle-meme — c'est le
    /// role du Garbage Collector, qui doit avoir prealablement confirme
    /// que le bloc n'est plus reference par aucun manifest vivant
    /// (docs/006-storage.md §13, invariant §22).
    pub fn delete_block(&self, id: &Uuid) -> Result<(), StorageError> {
        let path = self.path_for(id);
        fs::remove_file(path)?;
        Ok(())
    }

    pub fn block_exists(&self, id: &Uuid) -> bool {
        self.path_for(id).exists()
    }
}

/// Decoupe un contenu en blocs de taille fixe (docs/006-storage.md §7 —
/// decision : taille fixe, pas de content-defined chunking en V1).
pub fn chunk_fixed_size(data: &[u8], chunk_size: usize) -> Vec<&[u8]> {
    assert!(chunk_size > 0, "chunk_size doit etre strictement positif");
    data.chunks(chunk_size).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("syfi-storage-test-{name}-{}", Uuid::new_v4()));
        p
    }

    #[test]
    fn write_then_read_roundtrip() {
        let dir = temp_dir("roundtrip");
        let store = BlockStore::open(&dir).unwrap();
        let nonce = [3u8; 24];
        let ciphertext = b"donnees chiffrees de test".to_vec();

        let id = store.write_block(BlockToStore { nonce, ciphertext: &ciphertext }).unwrap();
        let (read_nonce, read_ciphertext) = store.read_block(&id).unwrap();

        assert_eq!(read_nonce, nonce);
        assert_eq!(read_ciphertext, ciphertext);
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn each_write_gets_a_distinct_id() {
        let dir = temp_dir("distinct-ids");
        let store = BlockStore::open(&dir).unwrap();
        let id1 = store.write_block(BlockToStore { nonce: [0u8; 24], ciphertext: b"a" }).unwrap();
        let id2 = store.write_block(BlockToStore { nonce: [0u8; 24], ciphertext: b"a" }).unwrap();
        // Meme contenu, deux ecritures : deux identifiants differents.
        // Preuve par construction de l'absence de deduplication (docs/006-storage.md §2).
        assert_ne!(id1, id2);
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn reading_unknown_id_fails() {
        let dir = temp_dir("unknown");
        let store = BlockStore::open(&dir).unwrap();
        let unknown = Uuid::new_v4();
        assert!(store.read_block(&unknown).is_err());
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn delete_removes_block() {
        let dir = temp_dir("delete");
        let store = BlockStore::open(&dir).unwrap();
        let id = store.write_block(BlockToStore { nonce: [0u8; 24], ciphertext: b"x" }).unwrap();
        assert!(store.block_exists(&id));
        store.delete_block(&id).unwrap();
        assert!(!store.block_exists(&id));
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn fixed_size_chunking_splits_correctly() {
        let data = vec![0u8; 10];
        let chunks = chunk_fixed_size(&data, 3);
        assert_eq!(chunks.len(), 4); // 3+3+3+1
        assert_eq!(chunks[3].len(), 1);
    }
}
