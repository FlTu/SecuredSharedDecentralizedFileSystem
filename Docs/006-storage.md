# docs/006-storage.md

# Storage Engine

Version : 2.1 (Draft — correction de cohérence avec `000-introduction.md`)

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

Pour lui, seules existent des séquences d'octets identifiées par un identifiant opaque.

---

# 2. Principes

Le moteur repose sur quatre principes.

## Immutable

Un bloc n'est jamais modifié.

Toute modification crée un nouveau bloc.

---

## Adressage par identifiant opaque

Chaque bloc est identifié par un **UUID généré à l'écriture**, pas par une empreinte de son contenu.

> Note de cohérence (V2.1) : la V2.0 de ce document décrivait un stockage *content addressable* avec déduplication globale. C'était incohérent avec `000-introduction.md`, qui place explicitement le *content-defined chunking* et la *déduplication globale* hors périmètre de la V1. Au-delà de la cohérence documentaire, l'adressage par contenu pose un problème de sécurité propre à ce projet : pour que deux blocs identiques produisent la même empreinte, il faudrait soit hasher le contenu chiffré (ce qui ne fonctionne pas avec un nonce aléatoire par bloc — deux chiffrements du même contenu produisent des empreintes différentes, la dédup ne se déclencherait jamais), soit adopter un chiffrement convergent à nonce déterministe dérivé du contenu en clair (ce qui permettrait à quiconque a accès au stockage physique de détecter que deux blocs contiennent le même contenu sans les déchiffrer, et expose à des attaques de confirmation de fichier). Ce compromis de sécurité n'a pas été demandé ; il est donc explicitement écarté. **Décision : pas de déduplication en V1, adressage par UUID opaque.**

---

## Append Only

Les nouvelles écritures sont ajoutées.

Aucune réécriture en place n'est autorisée.

---

## Crash Safe

Toute opération est transactionnelle.

Le WAL garantit la récupération.

---

# 3. Flux d'écriture

```
Vault

↓

Chunker (taille fixe)

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
BlockID (UUID v4, généré à l'écriture — jamais dérivé du contenu)

Size

Hash (intégrité uniquement — cf. §10, jamais utilisé pour l'adressage ou la dédup)

Algorithm

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

**Taille fixe, décidée pour la V1** (cf. architecture générale, section chiffrement) — priorité donnée à la simplicité et à la performance plutôt qu'à l'optimisation de la déduplication.

Le découpage par contenu (*content-defined chunking*, rolling-hash) reste hors périmètre V1, conformément à `000-introduction.md`. Il pourra être réévalué en V2 comme amélioration future, avec le compromis de sécurité qu'il implique (§2) explicitement assumé à ce moment-là — pas par défaut.

Le Storage n'impose pas la valeur exacte de la taille (paramètre de configuration du Vault/Chunker), mais elle est fixe pour un coffre donné, pas variable bloc par bloc.

---

# 8. Écriture

Une écriture suit toujours le même ordre.

```
Créer transaction

↓

Générer BlockID (UUID)

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

Il n'y a pas d'étape de recherche de doublon avant écriture : chaque import produit de nouveaux blocs avec de nouveaux identifiants, sans consultation d'un index de contenu.

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

Le hash de contenu (§4) sert exclusivement à cette vérification d'intégrité — jamais à l'adressage ni à la détection de doublons.

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

Le GC supprime les blocs qui ne sont plus référencés par aucun manifest vivant.

Cycle :

```
Scan Manifest

↓

Détection des BlockID non référencés

↓

Liste des blocs orphelins

↓

Suppression

↓

Compactage éventuel
```

Sans déduplication, chaque bloc n'appartient qu'à une seule entrée de manifest à la fois (pas de comptage de références partagées à maintenir) : un bloc devient orphelin dès que l'entrée qui le référence est remplacée par une nouvelle version ou supprimée (tombstone, cf. architecture générale).

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

# 16. Intégrité

Chaque lecture vérifie :

- checksum ;
- taille ;
- version ;
- format.

Les erreurs sont immédiatement signalées.

---

# 17. Compression

Le Storage reçoit des blocs déjà compressés si cette fonctionnalité est activée.

Il ne choisit jamais l'algorithme de compression.

Cette décision appartient au Vault.

---

# 18. Chiffrement

Le Storage ne connaît ni les clés ni les algorithmes.

Il manipule uniquement des données opaques.

---

# 19. Concurrence

Plusieurs lectures peuvent être exécutées simultanément.

Les écritures sont sérialisées par coffre afin de préserver la cohérence des transactions.

Des optimisations pourront être introduites dans une version ultérieure.

---

# 20. Erreurs

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

# 21. Observabilité

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

# 22. Invariants

Les règles suivantes sont absolues.

- Un bloc valide est immuable.
- Un `BlockID` est un UUID généré à l'écriture, jamais dérivé du contenu.
- Une écriture est toujours précédée par une entrée WAL.
- Une transaction n'est visible qu'après validation.
- Un bloc référencé par un manifest vivant ne peut jamais être supprimé.
- Le Storage ne modifie jamais le Manifest.
- Le Storage ne déchiffre jamais les données.
- Le Storage ne connaît jamais la notion de fichier.
- Le Storage ne réalise aucune détection ou déduplication de contenu.

---

# Conclusion

Le Storage Engine constitue la couche de persistance de SyFi.

En limitant strictement ses responsabilités au stockage de blocs immuables, transactionnels et adressés par identifiant opaque — sans déduplication ni adressage par contenu, conformément au périmètre V1 défini dans `000-introduction.md` — il offre une base robuste et tolérante aux pannes, sans introduire de canal d'observation involontaire sur le contenu stocké.
