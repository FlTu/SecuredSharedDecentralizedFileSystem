# docs/014-file-format.md

# Format de fichier

Version : 2.0 (Draft)

---

# 1. Objectif

Ce document spécifie le format physique et sur le fil de SyFi, condition nécessaire à l'exigence d'ouverture du format (`000-introduction.md` §3, `013-security.md` §8) : un utilisateur doit pouvoir récupérer ses données à partir de la seule documentation officielle et de sa passphrase, sans dépendre de l'application.

---

# 2. Organisation physique du coffre

Cf. `006-storage.md` §5 :

```
vault/
    blocks/
    manifest/
    wal/
    cache/
    tmp/
    config/
```

`cache/` et `tmp/` ne contiennent aucune donnée durable et peuvent être vidés sans perte.

---

# 3. Format d'un bloc

```
Header {
    magic: [u8; 4]          // identifiant de format, ex. "SYFB"
    format_version: u16
    block_id: [u8; 16]      // UUID v4, généré à l'écriture (cf. 006-storage.md §2)
    payload_size: u32
    encryption_algo: u8     // enum, ex. 1 = XChaCha20-Poly1305
    nonce: [u8; 24]
}

Payload {
    ciphertext: [u8; payload_size]
    auth_tag: [u8; 16]
}

Footer {
    integrity_hash: [u8; 32]   // BLAKE3, vérification uniquement (cf. 013-security.md §7)
}
```

Encodage little-endian pour tous les champs numériques multi-octets.

---

# 4. Format du manifest

Sérialisation **CBOR** (cf. `008-manifest.md` §5 pour la structure logique Header/Partitions/Journal/Merkle Index/Version Vectors/Footer).

## 4.1 Entrée d'index (`Node`)

```
IndexEntry {
    node_id: [u8; 16],          // UUID v7
    parent_id: [u8; 16],
    path: String,               // chemin logique, en clair uniquement dans le manifest déchiffré
    entry_type: u8,              // 0 = File, 1 = Directory
    size: u64,
    content_hash: [u8; 32],      // BLAKE3 du contenu en clair, intégrité uniquement
    blocks: Vec<BlockRef>,
    mtime: u64,
    version_vector: Map<DeviceId, u64>,
    deleted: bool,                // tombstone, cf. 009-sync.md §9
    deleted_at: Option<u64>,      // horodatage de suppression, pour le calcul de rétention
}

BlockRef {
    block_id: [u8; 16],   // UUID, correspond au BlockID du Storage (§3)
    nonce: [u8; 24],
    block_hash: [u8; 32], // intégrité uniquement
}
```

## 4.2 Header du manifest

```
ManifestHeader {
    manifest_format_version: u16,
    vault_id: [u8; 16],
    created_at: u64,
    last_update: u64,
    partition_count: u32,
    flags: u32,
}
```

---

# 5. Enveloppe de chiffrement

## 5.1 Clés dérivées par contexte

À partir de la clé maître du coffre (dérivée par Argon2id de la passphrase), deux sous-clés distinctes sont dérivées par HKDF-SHA256 :

```
manifest_key = HKDF(master_key, info = "syfi-manifest-v1")
blocks_key   = HKDF(master_key, info = "syfi-blocks-v1")
```

## 5.2 Enveloppe de distribution du manifest (hybride)

Cf. `013-security.md` §5 pour la justification. Format sur le fil :

```
ManifestEnvelope {
    format_version: u16,
    session_key_wrapped: Vec<SealedKeyForPeer>,  // une entrée par pair autorisé
    manifest_ciphertext: Vec<u8>,                 // XChaCha20-Poly1305(session_key, manifest_cbor)
    nonce: [u8; 24],
}

SealedKeyForPeer {
    recipient_device_id: [u8; 16],
    sealed_session_key: [u8; 48],  // sealed box X25519, 32 octets de clé + overhead
}
```

---

# 6. Identifiants

| Objet | Format |
|---|---|
| `BlockID` | UUID v4 |
| `NodeId` | UUID v7 (ordonnable temporellement, cf. `007-vault.md` §5) |
| Identifiant partagé (compte) | en-tête compte + corps appareil, encodage base58, versionné (cf. `011-identity.md` §4) |

---

# 7. Versionnement de format

Chaque composant possède une version indépendante (cf. `003-architecture.md` Partie 4, §45) :

```
Vault Format
Manifest Format
Block Format
IPC Protocol
Network Protocol
Identity Format
```

Règle de compatibilité : une nouvelle version doit pouvoir **lire** un format ancien (compatibilité ascendante) ; l'inverse n'est jamais garanti.

---

# 8. Outil de déchiffrement de référence

Livrable indépendant de l'application principale, à maintenir en parallèle de toute évolution de ce format :

- Implémentation minimale, dans un langage largement disponible (ex. Python avec bindings libsodium), capable de : dériver les clés à partir d'une passphrase, déchiffrer le manifest, déchiffrer et réassembler un fichier à partir de ses blocs.
- Sert à la fois de preuve que le format est réellement ouvert, et de garde-fou contre toute dérive accidentelle vers un format propriétaire de fait.

---

# 9. Conventions d'encodage

- CBOR pour toutes les structures sérialisées (manifest, enveloppes, messages IPC).
- Base58 pour les identifiants destinés à être partagés/affichés à l'utilisateur (identifiant de compte).
- UTF-8 pour tout texte en clair — jamais présent sur le disque en dehors du manifest déchiffré en mémoire.
- Little-endian pour tous les champs binaires numériques.

---

# Conclusion

Ce format constitue le contrat entre SyFi et toute implémentation indépendante. Toute modification doit être rétrocompatible en lecture et accompagnée d'une mise à jour correspondante de l'outil de déchiffrement de référence (§8).
