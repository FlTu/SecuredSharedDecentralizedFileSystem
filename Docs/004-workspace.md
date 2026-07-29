# docs/004-workspace.md

# Organisation du Workspace Rust

Version : 2.0 (Draft)

---

# 1. Objectif

Ce document définit l'organisation du dépôt Git ainsi que les règles de découpage des crates.

L'objectif est de garantir :

- une architecture modulaire ;
- une compilation rapide ;
- une forte testabilité ;
- un faible couplage ;
- une évolution simple.

L'organisation décrite ici constitue la structure officielle du projet.

---

# 2. Organisation générale

```
syfi/

├── Cargo.toml
├── Cargo.lock
│
├── crates/
│
├── desktop/
│
├── android/
│
├── cli/
│
├── docs/
│
├── tests/
│
├── examples/
│
├── tools/
│
├── scripts/
│
└── .github/
```

Chaque dossier possède une responsabilité unique.

---

# 3. Le Workspace Cargo

Le projet utilise un unique Workspace.

```
[workspace]

members = [

    "crates/*",

    "cli",

    "desktop",

]

resolver = "2"
```

Toutes les dépendances communes sont centralisées.

---

# 4. Philosophie du découpage

Contrairement à une architecture classique, chaque moteur est lui-même organisé en couches.

Chaque crate suit la structure :

```
Domain

↓

Application

↓

Infrastructure
```

Jamais l'inverse.

---

# 5. Les couches

## Domain

Le Domain contient uniquement :

- les règles métier ;
- les entités ;
- les Value Objects ;
- les invariants.

Le Domain ne dépend d'aucune bibliothèque externe.

Il ne connaît pas :

- Tokio
- libp2p
- SQLite
- le système de fichiers

---

## Application

Cette couche orchestre les cas d'utilisation.

Exemple :

```
CreateVault

OpenVault

ImportFile

DeleteFile

SynchronizeVault
```

Elle appelle le Domain.

Elle ne connaît pas l'infrastructure.

---

## Infrastructure

Cette couche contient les implémentations concrètes.

Exemples :

Filesystem

libp2p

SQLite

CBOR

IPC

Android

Windows

Linux

---

# 6. Arborescence des crates

```
crates/

    common/

    crypto/

    storage/

    manifest/

    vault/

    sync/

    network/

    identity/

    daemon/

    ipc/

    ffi/
```

---

# 7. Le crate Common

Le crate Common est volontairement minimal.

Il contient uniquement :

```
types

errors

events

ids

traits

macros

constants
```

Il ne contient aucune logique métier.

---

# 8. Le crate Crypto

Responsabilités :

- Argon2id
- HKDF
- XChaCha20
- Ed25519
- X25519
- RNG
- Zeroization

Le crate Crypto ne connaît jamais les coffres.

---

# 9. Le crate Storage

Responsabilités :

- blocs
- cache
- WAL
- GC
- lecture
- écriture

Le Storage ignore totalement les utilisateurs.

---

# 10. Le crate Manifest

Responsabilités :

- index

- partitions

- Merkle

- Version Vectors

- CBOR

Le Manifest ignore le disque.

---

# 11. Le crate Vault

Le Vault représente la logique métier.

Il expose :

```
CreateVault

OpenVault

CloseVault

Import

Export

Move

Rename

Delete
```

Le Vault ne réalise aucun chiffrement.

---

# 12. Le crate Sync

Responsabilités :

- planification
- reprise
- résolution
- téléchargement
- upload

Le Sync n'accède jamais directement au stockage.

---

# 13. Le crate Network

Responsabilités :

- transport
- découverte
- sessions
- relais
- NAT traversal

Aucune logique métier.

---

# 14. Le crate Identity

Responsabilités :

- comptes
- appareils
- certificats
- signatures
- QR Code

---

# 15. Le crate Daemon

Le Daemon est le point d'entrée du système.

Il orchestre :

- les coffres
- les identités
- le réseau
- la synchronisation
- l'IPC

Il ne contient aucune logique métier.

---

# 16. Le crate IPC

Expose une API stable.

Cette API est la seule officiellement supportée.

Toutes les applications passent par elle.

---

# 17. Le crate FFI

Permet une intégration future avec :

- Kotlin
- Swift
- C
- C++

Il ne contient aucune logique métier.

---

# 18. Les adaptateurs

Chaque technologie externe possède son propre adaptateur.

```
FilesystemAdapter

Libp2pAdapter

SQLiteAdapter

TokioAdapter

AndroidAdapter

DesktopAdapter
```

Les adaptateurs ne sont jamais utilisés directement par le Domain.

---

# 19. Organisation interne d'une crate

Exemple :

```
vault/

src/

domain/

application/

infrastructure/

api/

errors.rs

lib.rs
```

Cette structure est identique pour toutes les crates.

---

# 20. Les interfaces

Toutes les communications utilisent des traits Rust.

Exemple :

```
StoragePort

NetworkPort

ManifestPort

IdentityPort
```

Le Domain ne dépend que de ces interfaces.

---

# 21. Les dépendances

Autorisées :

```
Vault

↓

Manifest

↓

Storage

↓

Crypto
```

Interdites :

```
Storage

↓

Vault
```

```
Network

↓

Storage
```

```
Manifest

↓

Network
```

Toute dépendance circulaire est interdite.

---

# 22. Runtime

Le runtime officiel est Tokio.

Cependant, aucun moteur ne dépend directement de Tokio.

Le runtime est injecté via des adaptateurs.

---

# 23. Event Bus

Le bus d'événements est partagé.

Tous les moteurs utilisent la même interface.

```
publish()

subscribe()

unsubscribe()
```

Aucune crate ne connaît les abonnés.

---

# 24. Logger

Chaque moteur possède son logger.

Format :

```
JSON

Correlation ID

Timestamp UTC

Module

Severity

Message
```

---

# 25. Tests

Chaque crate possède :

```
tests/

unit/

integration/

bench/
```

Les benchmarks utilisent Criterion.

---

# 26. Documentation

Chaque crate contient :

```
README.md

ARCHITECTURE.md

CHANGELOG.md
```

Les API publiques utilisent systématiquement la documentation Rust (`///`).

---

# 27. Politique des dépendances

Les crates externes sont limitées.

Les dépendances transitoires doivent être surveillées.

Une dépendance ne peut être ajoutée sans justification.

---

# 28. Versionnement

Chaque crate possède sa propre version.

Le workspace possède également une version globale.

Exemple :

```
Workspace

1.2.0

Storage

1.4.0

Crypto

2.1.0

Daemon

1.3.2
```

---

# 29. Conventions

Tous les crates suivent les mêmes conventions.

- rustfmt obligatoire
- clippy obligatoire
- documentation publique obligatoire
- zéro warning
- zéro unsafe (sauf justification documentée)

---

# 30. Conclusion

Cette organisation permet de maintenir un cœur applicatif indépendant de toute technologie particulière.

Les composants restent testables, remplaçables et faiblement couplés.

Le Workspace constitue la base officielle de l'implémentation de SyFi.