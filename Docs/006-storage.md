# docs/006-storage.md

# Storage Engine

Version : 2.0 (Draft)

---

# 1. Objectif

Le Storage Engine est responsable du stockage physique des données.

Il constitue la couche la plus basse du Core.

Ses responsabilités sont limitées à :

- stocker des blocs ;
- relire des blocs ;
- vérifier leur intégrité ;
- gérer le cache ;
- gérer le WAL ;
- supprimer les blocs inutilisés.

Le Storage ne connaît jamais :

- les fichiers ;
- les dossiers ;
- les utilisateurs ;
- les permissions ;
- les identités.

Pour lui, seules existent des séquences d'octets identifiées par leur contenu.

---

# 2. Principes

Le moteur repose sur cinq principes.

## Immutable

Un bloc n'est jamais modifié.

Toute modification crée un nouveau bloc.

---

## Content Addressable

Chaque bloc est identifié par son empreinte cryptographique.

```
Hash

↓

Identifiant du bloc
```

---

## Append Only

Les nouvelles écritures sont ajoutées.

Aucune réécriture en place n'est autorisée.

---

## Crash Safe

Toute opération est transactionnelle.

Le WAL garantit la récupération.

---

## Déduplication

Deux contenus identiques partagent le même bloc.

---

# 3. Flux d'écriture

```
Vault

↓

Chunker

↓

Crypto

↓

Storage

↓

WAL

↓

Bloc Persisté

↓

Manifest
```

Le Storage ne reçoit que des blocs chiffrés.

---

# 4. Structure logique

Chaque bloc est représenté par :

```
BlockID

Size

Hash

Algorithm

Compression

Encryption

Checksum

Flags
```

Le contenu n'est jamais interprété.

---

# 5. Organisation physique

Le coffre contient plusieurs espaces.

```
vault/

blocks/

manifest/

wal/

cache/

tmp/

config/
```

Chaque répertoire possède une responsabilité unique.

---

# 6. Les blocs

Chaque bloc possède :

```
Header

Payload

Footer
```

Le Header contient uniquement des métadonnées techniques.

Le Payload contient les données chiffrées.

Le Footer contient les sommes de contrôle.

---

# 7. Taille des blocs

Le Chunker fournit des blocs de taille variable.

Objectifs :

- optimiser la déduplication ;
- réduire les transferts ;
- limiter la fragmentation.

Le Storage n'impose aucune taille.

---

# 8. Écriture

Une écriture suit toujours le même ordre.

```
Créer transaction

↓

Écrire WAL

↓

Écrire bloc

↓

fsync()

↓

Valider

↓

Supprimer WAL
```

Cette séquence garantit qu'aucun bloc partiellement écrit n'est considéré comme valide.

---

# 9. Lecture

```
BlockID

↓

Lookup

↓

Lecture disque

↓

Checksum

↓

Retour
```

Le Storage ne déchiffre jamais les données.

---

# 10. Vérification

Avant tout retour :

- checksum ;
- taille ;
- cohérence du header.

Toute anomalie entraîne une erreur.

---

# 11. Cache

Le Storage possède un cache mémoire.

Le cache contient uniquement :

- blocs récemment lus ;
- blocs récemment écrits ;
- index.

Les blocs volumineux ne restent pas durablement en mémoire.

---

# 12. Politique de cache

Le cache est limité.

Une politique LRU est utilisée dans la V1.

Une autre stratégie pourra être introduite ultérieurement.

---

# 13. Garbage Collector

Le GC supprime les blocs non référencés.

Cycle :

```
Scan Manifest

↓

Détection

↓

Liste des blocs orphelins

↓

Suppression

↓

Compactage éventuel
```

Le GC ne s'exécute jamais pendant une transaction active.

---

# 14. Journal transactionnel

Chaque transaction possède :

```
TransactionID

Timestamp

Operations

Status
```

Les états possibles sont :

```
Pending

Committed

RolledBack
```

---

# 15. Récupération

Au démarrage :

```
Lire WAL

↓

Transaction incomplète ?

↓

Oui

↓

Rollback ou Finalisation
```

Le système doit pouvoir récupérer sans intervention de l'utilisateur.

---

# 16. Déduplication

Avant d'écrire un bloc :

```
Hash

↓

Existe ?

↓

Oui

↓

Incrément du compteur de références

↓

Non

↓

Écriture
```

La déduplication est transparente.

---

# 17. Références

Le Storage maintient un compteur de références par bloc.

```
Bloc

↓

RefCount
```

Un bloc ne peut être supprimé que lorsque son compteur atteint zéro.

---

# 18. Intégrité

Chaque lecture vérifie :

- checksum ;
- taille ;
- version ;
- format.

Les erreurs sont immédiatement signalées.

---

# 19. Compression

Le Storage reçoit des blocs déjà compressés si cette fonctionnalité est activée.

Il ne choisit jamais l'algorithme de compression.

Cette décision appartient au Vault.

---

# 20. Chiffrement

Le Storage ne connaît ni les clés ni les algorithmes.

Il manipule uniquement des données opaques.

---

# 21. Concurrence

Plusieurs lectures peuvent être exécutées simultanément.

Les écritures sont sérialisées par coffre afin de préserver la cohérence des transactions.

Des optimisations pourront être introduites dans une version ultérieure.

---

# 22. Erreurs

Les erreurs principales sont :

```
BlockNotFound

InvalidChecksum

CorruptedHeader

WriteFailure

ReadFailure

DiskFull

PermissionDenied

InvalidFormat
```

Toutes héritent de `StorageError`.

---

# 23. Observabilité

Le moteur publie notamment :

```
BlockStored

BlockRead

CacheHit

CacheMiss

GCStarted

GCCompleted

TransactionCommitted

TransactionRolledBack
```

Ces événements sont diffusés via l'Event Bus.

---

# 24. Invariants

Les règles suivantes sont absolues.

- Un bloc valide est immuable.
- Une écriture est toujours précédée par une entrée WAL.
- Une transaction n'est visible qu'après validation.
- Un bloc référencé ne peut jamais être supprimé.
- Le Storage ne modifie jamais le Manifest.
- Le Storage ne déchiffre jamais les données.
- Le Storage ne connaît jamais la notion de fichier.

---

# Conclusion

Le Storage Engine constitue la couche de persistance de SyFi.

En limitant strictement ses responsabilités au stockage de blocs immuables et transactionnels, il offre une base robuste, dédupliquée et tolérante aux pannes sur laquelle reposent l'ensemble des moteurs supérieurs.