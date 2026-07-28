# Architecture — Application de partage de fichiers chiffré P2P

*Nom de code : à définir. Document de conception initiale.*

## 1. Objectifs et périmètre

Créer une application qui :

- présente un **répertoire virtuel** reconstruit à partir d'un **stockage physique chiffré** (façon cryfs/gocryptfs) ;
- **synchronise** ce répertoire entre plusieurs appareils appartenant à un même utilisateur, ou partagés avec des contacts, via un réseau **pair-à-pair** sans serveur central obligatoire ;
- s'identifie via une **paire de clés cryptographiques** (pas de compte, pas d'e-mail, pas de serveur d'authentification) ;
- se **compile depuis un unique poste Linux** vers trois cibles : Linux, Windows, Android.

Hors périmètre pour la V1 (à réévaluer plus tard) : partage à des groupes larges (>10 pairs actifs simultanés), résolution de conflits automatique intelligente au-delà du "garder les deux versions", interface web/iOS, déduplication inter-fichiers.

## 2. Vue d'ensemble en couches

```
┌─────────────────────────────────────────────────────────┐
│  UI (par plateforme)                                     │
│  - Linux/Windows : Tauri (webview) ou egui (natif)        │
│  - Android : Kotlin, appel du core via JNI                │
├─────────────────────────────────────────────────────────┤
│  Core Rust (une seule codebase, compilée sur les 3 cibles)│
│  ┌───────────────┐ ┌───────────────┐ ┌─────────────────┐ │
│  │ Vault Engine   │ │ Index Engine  │ │ Sync Engine     │ │
│  │ (chiffrement,  │ │ (arborescence,│ │ (comparaison    │ │
│  │  blocs, clés)  │ │  manifeste)   │ │  Merkle, diff)  │ │
│  └───────────────┘ └───────────────┘ └─────────────────┘ │
│  ┌───────────────────────────────────────────────────────┐│
│  │ Network Engine (libp2p) : identité, découverte,        ││
│  │ transport chiffré, transfert de blocs                  ││
│  └───────────────────────────────────────────────────────┘│
├─────────────────────────────────────────────────────────┤
│  Stockage physique local (fichiers opaques sur disque)     │
└─────────────────────────────────────────────────────────┘
```

Le Core Rust ne dépend d'aucune UI. C'est une bibliothèque (`cdylib` + `staticlib`) exposant une API C stable (FFI), consommée directement par Tauri/egui sur desktop et via `jni` sur Android. Toute la logique sensible (crypto, sync, réseau) vit à un seul endroit : aucune réimplémentation par plateforme.

## 3. Vault Engine — le volume chiffré

### 3.1 Découpage et chiffrement

- Chaque fichier logique est découpé en **blocs de taille fixe** (proposition : 4 Mio, ajustable).
- Chaque bloc est chiffré indépendamment en **XChaCha20-Poly1305** :
  - nonce 192 bits généré aléatoirement par bloc (espace assez grand pour exclure toute collision pratique, contrairement à AES-GCM en 96 bits) ;
  - pas de dépendance à une accélération matérielle AES, donc performance homogène desktop/mobile.
- Clé de chiffrement des blocs dérivée par **HKDF-SHA256** à partir d'une clé maître (elle-même dérivée d'un mot de passe utilisateur via **Argon2id**, paramètres à durcir selon le matériel cible).
- Chaque bloc chiffré est stocké physiquement sous un **nom opaque** (UUID v4 ou hash du contenu chiffré) — le nom réel du fichier et sa position dans l'arborescence n'existent **que** dans l'index chiffré (cf. §4). Ce point est un choix de conception fort : il protège les métadonnées (comme cryfs) au prix d'une indirection systématique.

### 3.2 Gestion des clés

- Une clé maître par "coffre" (vault), stockée chiffrée par un mot de passe local (Argon2id + XChaCha20-Poly1305).
- Le partage d'un coffre avec un contact = partage hors-bande de la clé maître (via un canal déjà authentifié : l'échange d'identité P2P du §5), pas de re-dérivation par mot de passe côté destinataire.
- Prévoir dès la V1 une possibilité de **rotation de clé** (même si non implémentée tout de suite) : ne pas coder en dur l'hypothèse "une seule clé pour toujours".

### 3.3 Montage / accès

- **Linux** : montage FUSE réel via le crate `fuser`. Le répertoire virtuel apparaît comme un vrai point de montage.
- **Windows** : montage virtuel via WinFsp (binding `winfsp-rs`), même principe.
- **Android** : **pas de FUSE réaliste sans root**. L'accès se fait via un explorateur de fichiers intégré à l'application, qui déchiffre à la demande (lecture/écriture de blocs individuels, pas de déchiffrement intégral en mémoire).

## 4. Index Engine — le manifeste

### 4.1 Rôle

L'index est l'unique source de vérité sur l'arborescence logique. Il est lui-même chiffré et versionné.

### 4.2 Structure (proposition, sérialisée en CBOR)

```
IndexEntry {
  path: String,              // chemin logique complet
  entry_type: File | Dir,
  size: u64,
  content_hash: [u8; 32],    // BLAKE3 du contenu en clair
  blocks: Vec<BlockRef>,     // liste ordonnée des blocs
  mtime: u64,
  version_vector: Map<PeerId, u64>,  // cf. §4.3
  deleted: bool,              // tombstone plutôt que suppression physique immédiate
}

BlockRef {
  block_id: Uuid,     // = nom physique opaque sur disque
  nonce: [u8; 24],
  block_hash: [u8; 32],
}
```

Choix : **CBOR plutôt que JSON** pour la compacité et la vitesse de parsing (l'index est lu/écrit très fréquemment), et parce qu'il supporte nativement les types binaires (hash, nonce) sans encodage base64 intermédiaire.

### 4.3 Versionnement et conflits

- Chaque entrée porte un **vecteur de version** (un compteur par PeerId ayant modifié le fichier), pas un simple timestamp — un timestamp seul ne permet pas de distinguer "modification concurrente" de "modification séquentielle", ce qui est le problème classique de tous les synchroniseurs naïfs.
- Politique de conflit V1 : si deux vecteurs de version sont **incomparables** (ni l'un ni l'autre ne domine l'autre), on conserve les deux fichiers, en renommant l'un des deux (`fichier (conflit sur <peer>, <date>).ext`), à charge pour l'utilisateur de trancher manuellement. C'est la politique la plus simple qui ne perd jamais de données silencieusement.

## 5. Network Engine — identité et transport

### 5.1 Identité

- Chaque appareil génère une paire de clés **Ed25519** au premier lancement.
- La **clé publique encodée** (base58 ou base32, façon adresse Tox/Session) est l'identifiant que l'utilisateur partage à ses contacts.
- Ajout de contact = échange mutuel hors-bande de cette clé publique (QR code entre deux appareils, ou lien copié/collé).

### 5.2 Transport et découverte

Construit sur **libp2p** (implémentation Rust mature), qui fournit directement :

- **Découverte locale** : mDNS pour les pairs sur le même réseau.
- **Découverte distante** : DHT Kademlia pour retrouver un pair par son identifiant public sur Internet.
- **Chiffrement de transport** : protocole Noise (indépendant du chiffrement du contenu du §3, défense en profondeur).
- **Traversée NAT** : relais + hole punching, pour les cas où les deux pairs sont derrière des box grand public.

Ce choix évite de réimplémenter ces briques (c'est tout l'historique de projets comme Tox ou Briar, non trivial à faire soi-même correctement).

### 5.3 Protocole de synchronisation (façon torrent)

1. Deux pairs connectés échangent la **racine d'un arbre de Merkle** construit sur les hashs des `IndexEntry`.
2. Si les racines diffèrent, ils descendent l'arbre pour isoler précisément **quelles entrées** diffèrent (pas besoin de comparer fichier par fichier un par un).
3. Pour chaque entrée divergente, comparaison des vecteurs de version → détermine qui a la version la plus récente, ou déclenche un conflit (§4.3).
4. Seuls les **blocs manquants** sont transférés (identifiés par leur `block_id`/hash) — et si le même dossier est partagé par plusieurs pairs, les blocs peuvent être récupérés en parallèle depuis plusieurs sources, comme un swarm torrent.

## 6. Cross-compilation depuis Linux

| Cible | Toolchain | Méthode |
|---|---|---|
| Linux x86_64 | native | `cargo build --release` |
| Windows x86_64 | `x86_64-pc-windows-gnu` | `cross` (via Docker) ou mingw-w64 installé localement |
| Android arm64 / armv7 | NDK | `cargo-ndk` |

Le Core Rust se compile en `cdylib` (`.so` pour Linux/Android, `.dll` pour Windows) consommée par la couche UI native de chaque plateforme. Un seul `Cargo.toml` workspace, avec les crates `vault-engine`, `index-engine`, `sync-engine`, `network-engine`, `ffi` séparées pour permettre de tester chaque brique indépendamment.

## 7. Plan de développement proposé (par phases)

1. **Vault Engine seul** : chiffrement/déchiffrement de blocs, tests unitaires, CLI minimale pour valider le format sur disque.
2. **Index Engine** : lecture/écriture du manifeste, gestion des vecteurs de version, toujours en local (pas de réseau).
3. **Squelette cross-compilé** : vérifier que le Core compile et s'exécute sur les 3 cibles avant d'ajouter la complexité réseau.
4. **Network Engine** : identité, découverte LAN (mDNS) en premier, DHT ensuite.
5. **Sync Engine** : protocole Merkle + transfert de blocs, d'abord entre deux pairs sur LAN.
6. **UI** par plateforme, en dernier, une fois le Core stable.

## 8. Points ouverts à trancher avant de coder

- Chiffrement des **noms de fichiers/métadonnées** (comme cryfs) ou contenu seul (comme gocryptfs simple) ? Impacte directement le format du §3.
- Taille de bloc fixe (simplicité) vs découpage par contenu façon rolling-hash (meilleure dédup sur fichiers modifiés partiellement, mais complexité accrue) ?
- Faut-il un mode "serveur relais optionnel" auto-hébergeable pour les cas où aucun hole-punching ne fonctionne, ou tout miser sur libp2p relay public ?
- Politique de rétention des tombstones (entrées supprimées) : purge après combien de temps ?