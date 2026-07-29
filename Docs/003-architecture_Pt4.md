# docs/003-architecture.md

# Architecture Générale

## Partie 4 — Extensibilité, Compatibilité et Évolution

Version : 2.0 (Draft)

---

# 41. Philosophie d'évolution

Une architecture n'est pas seulement conçue pour répondre aux besoins actuels.

Elle doit également permettre d'intégrer des fonctionnalités futures sans remettre en cause les fondations.

SyFi est conçu selon le principe suivant :

> **Toute évolution doit être additive avant d'être disruptive.**

Autrement dit :

- une nouvelle fonctionnalité ne doit pas casser les anciennes ;
- un nouveau format doit pouvoir cohabiter avec les précédents ;
- un nouveau module ne doit pas imposer une refonte du Core.

---

# 42. Architecture hexagonale (Ports & Adapters)

Le Core ne dépend d'aucune technologie particulière.

Il ne connaît ni :

- libp2p ;
- Tauri ;
- Android ;
- le système de fichiers ;
- SQLite ;
- FUSE.

Le Core dépend uniquement d'interfaces abstraites (Ports).

Les implémentations concrètes sont des adaptateurs.

```
               +----------------------+
               |     Applications     |
               +----------+-----------+
                          |
                    IPC / API
                          |
+------------------------------------------------+
|                 Core SyFi                      |
|                                                |
|  Vault  Manifest  Sync  Identity  Crypto       |
|                                                |
|        Ports (Traits Rust)                     |
+------------------------------------------------+
        |        |          |         |
        v        v          v         v
     Filesystem  libp2p    SQLite   Tauri
       Adapter   Adapter   Adapter  Adapter
```

Cette approche permet de remplacer une technologie sans modifier la logique métier.

---

# 43. Définition des Ports

Chaque moteur expose un ensemble limité d'interfaces.

Exemple :

```
StoragePort

ManifestPort

NetworkPort

IdentityPort

EventBusPort

LoggerPort
```

Les crates du Core ne dépendent que de ces interfaces.

---

# 44. Les Adaptateurs

Les adaptateurs relient le monde extérieur au Core.

Exemples :

```
FilesystemAdapter

Libp2pAdapter

TokioRuntimeAdapter

IPCAdapter

AndroidAdapter

DesktopAdapter
```

Un adaptateur ne contient aucune logique métier.

Son rôle est uniquement de traduire une API externe vers un Port du Core.

---

# 45. Versionnement

Chaque élément du système possède une version indépendante.

```
Vault Format

Manifest Format

IPC Protocol

Network Protocol

Identity Format

Configuration Format
```

Chaque numéro de version est propre à son domaine.

---

# 46. Compatibilité

Les règles sont les suivantes.

## Lecture

Une nouvelle version doit pouvoir ouvrir :

- un coffre ancien ;
- un manifest ancien ;
- une configuration ancienne.

---

## Écriture

Une ancienne version n'a aucune obligation d'ouvrir un coffre plus récent.

La compatibilité est donc :

→ ascendante

et non descendante.

---

# 47. Migration

Les migrations sont explicites.

Chaque migration possède :

- une version source ;
- une version cible ;
- une procédure de validation ;
- une procédure de rollback lorsque possible.

Exemple :

```
Manifest v3

↓

Migration

↓

Manifest v4
```

---

# 48. Dépréciation

Aucune fonctionnalité ne disparaît brutalement.

Cycle de vie :

```
Stable

↓

Deprecated

↓

Legacy

↓

Removed
```

Une fonctionnalité dépréciée reste documentée.

---

# 49. Architecture des plugins

La V1 ne fournit pas de système de plugins.

Cependant, toute l'architecture doit permettre leur ajout.

Les plugins ne pourront jamais :

- accéder directement aux structures internes ;
- modifier le manifest ;
- écrire dans le stockage.

Ils devront passer par l'IPC publique.

---

# 50. Événements publics

Le démon expose un Event Bus public.

Exemples :

```
VaultOpened

VaultClosed

SyncStarted

SyncCompleted

PeerConnected

PeerDisconnected

DeviceAdded

IdentityImported
```

Les applications peuvent s'abonner à ces événements.

---

# 51. API IPC

L'IPC constitue la seule API officiellement supportée.

Les crates Rust ne sont pas considérées comme une API publique.

Cela permet de faire évoluer le Core sans casser les applications.

---

# 52. ADR (Architecture Decision Records)

Toute décision structurante est documentée.

Chaque ADR comprend :

- contexte ;
- problème ;
- solutions envisagées ;
- décision retenue ;
- conséquences.

Exemple :

```
ADR-0001

Daemon Central
```

```
ADR-0002

Manifest Partitionné
```

```
ADR-0003

Event Bus
```

---

# 53. Politique de dépendances

Les dépendances externes sont limitées.

Chaque nouvelle bibliothèque doit répondre aux critères suivants :

- maintenue activement ;
- licence compatible ;
- communauté active ;
- auditée lorsque critique.

Les dépendances cryptographiques doivent être minimales.

---

# 54. Remplacement d'un composant

Chaque moteur doit pouvoir être remplacé indépendamment.

Exemple :

Aujourd'hui :

```
Network

↓

libp2p
```

Demain :

```
Network

↓

QUIC natif
```

Le reste du système ne change pas.

---

# 55. Évolutions prévues

L'architecture doit permettre l'ajout de :

- synchronisation sélective ;
- snapshots ;
- historique complet ;
- recherche locale ;
- stockage objet ;
- backend S3 ;
- backend IPFS ;
- backend WebDAV ;
- synchronisation serveur facultative ;
- moteur de collaboration ;
- messagerie ;
- appels audio ;
- appels vidéo.

Aucun de ces ajouts ne doit nécessiter une réécriture du Core.

---

# 56. Interopérabilité

Les formats de données sont ouverts.

Une implémentation indépendante doit pouvoir :

- ouvrir un coffre ;
- lire les blocs ;
- vérifier les signatures ;
- reconstruire les fichiers.

Sans dépendre du code officiel.

---

# 57. Écosystème

Le démon est considéré comme un système d'exploitation miniature.

Il fournit plusieurs services :

```
Identity Service

Vault Service

Sync Service

Network Service

Notification Service

IPC Service

Configuration Service
```

Toutes les futures applications utiliseront ces mêmes services.

---

# 58. Règles d'évolution

Toute évolution devra respecter les règles suivantes.

✓ Ne jamais casser un coffre existant.

✓ Ne jamais casser un protocole sans version.

✓ Ne jamais casser une API sans remplacement.

✓ Documenter chaque décision.

✓ Ajouter avant de supprimer.

✓ Versionner chaque format.

---

# 59. Objectif à long terme

À terme, SyFi ne sera plus uniquement une application de synchronisation.

Il deviendra une plateforme distribuée capable d'héberger plusieurs applications partageant :

- les identités ;
- les connexions réseau ;
- la cryptographie ;
- la synchronisation ;
- les politiques de sécurité.

Le partage de fichiers sera simplement la première application construite sur cette plateforme.

---

# Conclusion

L'architecture de SyFi est conçue pour évoluer sur le long terme.

Les technologies employées aujourd'hui pourront être remplacées demain sans remettre en cause les fondations du système.

Cette indépendance constitue l'un des objectifs majeurs du projet.