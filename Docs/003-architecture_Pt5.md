# docs/003-architecture.md

# Architecture Générale

## Partie 5 — Architecture finale, dépendances et conventions

Version : 2.0 (Draft)

---

# 60. Architecture logique finale

L'architecture complète de SyFi est organisée selon le modèle suivant.

```
┌───────────────────────────────────────────────────────┐
│                   Applications                        │
│                                                       │
│ Desktop │ Android │ CLI │ Future Messenger │ Plugins │
└──────────────────────────┬────────────────────────────┘
                           │
                           │ IPC
                           ▼
┌───────────────────────────────────────────────────────┐
│                     Daemon                            │
│                                                       │
│ Session Manager                                       │
│ Capability Manager                                    │
│ Scheduler                                              │
│ Event Bus                                              │
│ Configuration                                           │
└───────────────┬───────────────────────────────────────┘
                │
                ▼
┌───────────────────────────────────────────────────────┐
│                     Core                              │
│                                                       │
│ Vault │ Manifest │ Sync │ Identity │ Network │ Crypto │
│                 │ Storage │                          │
└───────────────────────────────────────────────────────┘
                │
                ▼
┌───────────────────────────────────────────────────────┐
│                     Adapters                          │
│                                                       │
│ Filesystem │ libp2p │ Tokio │ IPC │ OS │ Logging      │
└───────────────────────────────────────────────────────┘
```

---

# 61. Dépendances autorisées

Les dépendances sont strictement contrôlées.

```
Application

↓

IPC

↓

Daemon

↓

Core

↓

Ports

↓

Adapters

↓

Système
```

Une dépendance inverse est interdite.

---

# 62. Dépendances entre crates

Le graphe suivant constitue la référence officielle.

```
crypto

↑

storage

↑

manifest

↑

vault

↑

sync

↑

daemon
```

En parallèle :

```
identity

↑

daemon
```

```
network

↑

daemon
```

Les moteurs ne doivent jamais créer de dépendances circulaires.

---

# 63. Arborescence du projet

```
syfi/

├── Cargo.toml

├── crates/

│   ├── crypto/
│   ├── storage/
│   ├── manifest/
│   ├── vault/
│   ├── sync/
│   ├── identity/
│   ├── network/
│   ├── daemon/
│   ├── ipc/
│   ├── ffi/
│   └── common/

├── desktop/

├── android/

├── cli/

├── docs/

├── tests/

├── examples/

└── tools/
```

Cette structure ne doit évoluer qu'en cas de nécessité démontrée.

---

# 64. Le crate Common

Le crate `common` contient uniquement :

- types partagés ;
- identifiants ;
- erreurs communes ;
- événements ;
- constantes ;
- traits publics.

Il ne contient aucune logique métier.

---

# 65. Les interfaces publiques

Chaque crate expose un nombre limité d'interfaces.

Exemple :

```
VaultService

StorageService

NetworkService

IdentityService

SyncService
```

Les structures internes restent privées.

---

# 66. Gestion des erreurs

Toutes les erreurs implémentent un trait commun.

```
SyFiError

├── StorageError

├── VaultError

├── SyncError

├── CryptoError

├── NetworkError

├── IdentityError

└── IPCError
```

Une erreur ne doit jamais traverser plusieurs couches sans être enrichie de contexte.

---

# 67. Journalisation

Chaque moteur possède son propre logger.

Exemple :

```
[Vault]

[Storage]

[Sync]

[Network]

[Daemon]
```

Les journaux sont structurés.

Le format JSON est privilégié pour faciliter l'analyse.

---

# 68. Configuration

Toute la configuration est centralisée.

Le démon est le seul responsable du chargement.

Les moteurs ne lisent jamais directement un fichier de configuration.

---

# 69. Gestion des ressources

Chaque moteur est responsable de ses propres ressources.

Exemple :

Storage :

- fichiers ouverts
- cache

Network :

- sockets
- connexions

Vault :

- handles actifs

Identity :

- clés en mémoire

Le démon ne manipule jamais directement ces ressources.

---

# 70. Observabilité

Le système doit permettre :

- la journalisation ;
- les métriques ;
- les traces ;
- le diagnostic.

Chaque opération importante reçoit un identifiant de corrélation (Correlation ID) permettant de suivre une requête de bout en bout.

Exemple :

```
Import File

↓

Transaction ID

↓

Logs

↓

Network

↓

Sync

↓

Storage

↓

Commit
```

---

# 71. Politique de tests

Chaque crate doit posséder :

- des tests unitaires ;
- des tests d'intégration ;
- des tests de performance lorsque nécessaire.

Le Core ne doit dépendre d'aucune interface graphique pour être testé.

---

# 72. Politique de documentation

Chaque API publique est documentée.

Chaque structure importante possède :

- sa description ;
- ses invariants ;
- ses contraintes.

Toute modification d'une API publique implique une mise à jour de la documentation.

---

# 73. Politique de sécurité

Aucune clé secrète n'est journalisée.

Aucun mot de passe n'est conservé en mémoire plus longtemps que nécessaire.

Les buffers contenant des données sensibles doivent être explicitement nettoyés lorsqu'ils ne sont plus utilisés.

---

# 74. Politique de performance

Les optimisations prématurées sont interdites.

Toute optimisation doit être justifiée par :

- une mesure ;
- un benchmark ;
- un besoin démontré.

La lisibilité prime tant que les objectifs de performance sont atteints.

---

# 75. Politique de revue de code

Toute contribution importante doit répondre aux questions suivantes :

- respecte-t-elle les invariants d'architecture ?
- introduit-elle un couplage supplémentaire ?
- est-elle testée ?
- est-elle documentée ?
- conserve-t-elle la compatibilité ?

---

# 76. Critères d'acceptation de l'architecture

L'architecture est considérée comme validée si :

✓ toutes les responsabilités sont clairement séparées ;

✓ aucune dépendance circulaire n'existe ;

✓ les formats sont versionnés ;

✓ les composants communiquent uniquement par leurs interfaces publiques ou par l'Event Bus ;

✓ chaque moteur peut être testé indépendamment ;

✓ chaque technologie externe est isolée derrière un adaptateur.

---

# Conclusion

Cette architecture constitue la référence officielle de SyFi.

Tous les développements devront s'y conformer.

Toute évolution structurante devra être documentée dans un ADR avant son implémentation.

L'objectif est de garantir une architecture stable, modulaire et pérenne, capable d'évoluer pendant de nombreuses années sans remettre en cause les données des utilisateurs.