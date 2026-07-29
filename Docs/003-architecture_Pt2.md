# docs/003-architecture.md

# Architecture Générale

## Partie 2 — Flux de données et interactions

---

# 10. Cycle de vie d'un fichier

Cette section décrit le parcours complet d'un fichier au sein de SyFi.

Le processus est identique quelle que soit la plateforme (Linux, Windows, Android).

L'interface utilisateur ne participe jamais directement au traitement.

---

## 10.1 Import

Le flux complet est le suivant.

```
Utilisateur

↓

Desktop / CLI / Android

↓

IPC

↓

Daemon

↓

Vault Engine

↓

Storage Engine

↓

Crypto Engine

↓

Manifest Engine

↓

Event Bus

↓

Sync Engine (notification uniquement)
```

---

### Étape 1

L'utilisateur sélectionne un fichier.

L'interface :

- ouvre le fichier ;
- envoie uniquement son chemin (ou son flux) au démon.

Aucune logique métier n'est exécutée côté interface.

---

### Étape 2

Le démon valide :

- le coffre ;
- les permissions ;
- la disponibilité du stockage ;
- l'espace disque.

Puis transmet la requête au Vault Engine.

---

### Étape 3

Le Vault Engine :

- lit le fichier ;
- découpe le contenu en blocs ;
- calcule les empreintes ;
- construit les métadonnées temporaires.

Le Vault ne chiffre jamais directement.

---

### Étape 4

Chaque bloc est envoyé au Crypto Engine.

Le Crypto Engine :

- génère le nonce ;
- chiffre le bloc ;
- calcule le tag d'authentification.

Il retourne :

```
Bloc chiffré

+

Métadonnées cryptographiques
```

---

### Étape 5

Le Storage Engine écrit :

- le bloc ;
- les métadonnées internes ;
- les informations nécessaires au WAL.

Aucune structure logique (répertoire, fichier, utilisateur) n'est connue du Storage.

---

### Étape 6

Le Manifest Engine met à jour :

- l'arborescence ;
- les références vers les blocs ;
- les versions ;
- les arbres Merkle ;
- les partitions concernées.

---

### Étape 7

Le Vault valide la transaction.

Le WAL est supprimé.

---

### Étape 8

Le démon publie :

```
FileAdded
```

Le Sync Engine est simplement informé.

Il ne démarre pas nécessairement une synchronisation.

---

# 11. Lecture d'un fichier

Le chemin inverse est utilisé.

```
Utilisateur

↓

Daemon

↓

Vault

↓

Manifest

↓

Storage

↓

Crypto

↓

Vault

↓

Daemon

↓

Application
```

---

Le Manifest indique :

- quels blocs lire ;
- dans quel ordre ;
- quelle version utiliser.

Le Storage lit les blocs.

Le Crypto les déchiffre.

Le Vault recompose le fichier.

Le démon renvoie le flux au client.

---

# 12. Suppression

La suppression ne détruit jamais immédiatement les blocs.

Le processus est :

```
Utilisateur

↓

Vault

↓

Manifest

↓

Tombstone

↓

Transaction

↓

Event Bus

↓

Garbage Collector (plus tard)
```

Les blocs deviennent simplement orphelins.

Le Garbage Collector décidera plus tard de leur suppression physique.

---

# 13. Déplacement

Déplacer un fichier est une opération purement logique.

Aucun bloc n'est réécrit.

Seules les structures suivantes évoluent :

- chemin
- parent
- index

Cette opération doit rester quasi instantanée.

---

# 14. Renommage

Même principe.

Aucun contenu n'est modifié.

Le Manifest met uniquement à jour le nom logique.

---

# 15. Synchronisation

Le Sync Engine est un orchestrateur.

Il ne lit jamais directement le disque.

Le flux est le suivant.

```
Event

↓

Sync Engine

↓

Manifest

↓

Comparaison

↓

Planification

↓

Network

↓

Storage

↓

Manifest

↓

Validation
```

---

## Étape 1

Le Sync reçoit :

```
VaultChanged
```

---

## Étape 2

Il compare :

Manifest Local

contre

Manifest Distant

Le résultat est une liste d'opérations.

---

## Étape 3

Le planificateur construit une file de tâches.

Exemple :

```
Télécharger :

Bloc A

Bloc B

Bloc C

Uploader :

Bloc D

Bloc E
```

---

## Étape 4

Les transferts commencent.

Ils peuvent être parallélisés.

Chaque téléchargement est indépendant.

---

## Étape 5

Chaque bloc reçu suit exactement le même pipeline que lors d'un import local.

```
Network

↓

Crypto

↓

Storage

↓

Manifest
```

Le Sync n'écrit jamais directement sur le disque.

---

# 16. Conflits

Le Sync ne résout jamais directement les conflits.

Son rôle est de :

- détecter
- classifier
- signaler

Le Vault décide ensuite de la politique applicable.

Cela permettra ultérieurement :

- stratégie Last Writer Wins ;
- fusion intelligente ;
- validation utilisateur ;
- plugins de résolution.

Sans modifier le moteur de synchronisation.

---

# 17. Réseau

Le Network Engine ne transporte que des messages.

Il ignore totalement :

- les fichiers ;
- les utilisateurs ;
- le contenu des blocs.

Il connaît uniquement :

- Peer
- Session
- Message
- Stream

---

# 18. Reprise après interruption

Toutes les opérations sont conçues pour être idempotentes.

Après un crash :

```
Ouverture coffre

↓

Lecture WAL

↓

Analyse

↓

Rollback

ou

Commit

↓

Reprise Event Bus

↓

Reprise Sync
```

L'utilisateur ne doit effectuer aucune action manuelle.

---

# 19. Transactions

Toutes les opérations modifiant le coffre utilisent la même structure.

```
Début transaction

↓

Journal WAL

↓

Écriture Storage

↓

Mise à jour Manifest

↓

Validation

↓

Suppression WAL

↓

Publication Event
```

Aucun événement n'est publié avant la validation complète.

Cette règle garantit que tous les consommateurs observent un état cohérent.

---

# 20. Invariants d'architecture

Les règles suivantes sont absolues.

Le Storage ne modifie jamais le Manifest.

Le Manifest n'accède jamais au disque.

Le Network ne connaît jamais le contenu des blocs.

Le Vault ne chiffre jamais directement.

Le Daemon ne contient aucune logique métier.

Les interfaces utilisateur ne manipulent jamais le Core.

Le Sync n'écrit jamais directement dans le stockage.

Le Crypto n'a aucune connaissance des fichiers.

Ces invariants constituent les fondations du projet.

Ils ne peuvent être modifiés qu'au travers d'un ADR.