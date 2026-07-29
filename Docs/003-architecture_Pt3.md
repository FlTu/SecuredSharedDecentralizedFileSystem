# docs/003-architecture.md

# Architecture Générale

## Partie 3 — États, Concurrence et Isolation

---

# 21. Principe général

SyFi est un système fortement concurrent.

À tout instant, plusieurs opérations peuvent être exécutées simultanément :

- lecture de fichiers ;
- import de nouveaux fichiers ;
- synchronisation distante ;
- découverte de nouveaux pairs ;
- garbage collection ;
- rotation des clés ;
- sauvegarde automatique.

L'architecture doit permettre cette exécution parallèle tout en garantissant :

- la cohérence des données ;
- l'absence de corruption ;
- la reproductibilité des opérations.

---

# 22. Machines d'état

Chaque composant critique est modélisé comme une machine d'état explicite.

Aucun composant ne doit reposer sur une succession implicite de booléens ou de drapeaux.

Les transitions doivent être exhaustives et vérifiables.

---

# 23. Machine d'état du Daemon

```
                +---------+
                |Stopped  |
                +---------+
                     |
                     v
              +-------------+
              |Initializing |
              +-------------+
                     |
                     v
                +---------+
                | Running |
                +---------+
               /     |     \
              /      |      \
             v       v       v
      Syncing   Idle Tasks   Shutdown Requested
             \       |       /
              \      |      /
               +------------+
               | Stopping   |
               +------------+
                     |
                     v
                +---------+
                |Stopped  |
                +---------+
```

Le démon ne peut jamais revenir directement de `Stopping` à `Running`.

---

# 24. Machine d'état d'un coffre

Chaque coffre possède son propre état.

```
Closed

↓

Opening

↓

Unlocked

↓

Busy

↓

Unlocked

↓

Closing

↓

Closed
```

Un coffre verrouillé ne peut jamais être synchronisé.

Un coffre en cours d'ouverture ne peut accepter aucune commande.

---

# 25. Machine d'état d'une synchronisation

```
Idle

↓

Discovery

↓

Negotiation

↓

Manifest Exchange

↓

Planning

↓

Transfer

↓

Verification

↓

Commit

↓

Completed
```

En cas d'erreur :

```
↓

Recovery

↓

Retry

↓

Failed
```

Toutes les transitions sont déterministes.

---

# 26. Sessions réseau

Chaque connexion avec un pair possède une session indépendante.

```
Disconnected

↓

Connecting

↓

Authenticated

↓

Negotiating

↓

Ready

↓

Streaming

↓

Closing

↓

Disconnected
```

Une erreur réseau ne doit jamais impacter les autres sessions.

---

# 27. États d'un bloc

Un bloc possède un cycle de vie interne.

```
Created

↓

Encrypted

↓

Stored

↓

Referenced

↓

Replicated

↓

Orphan

↓

Collected
```

Le Garbage Collector ne traite que les blocs `Orphan`.

---

# 28. États d'un manifest

```
Loaded

↓

Dirty

↓

Serializing

↓

Persisted

↓

Clean
```

Un manifest `Dirty` ne peut jamais être utilisé pour une synchronisation.

---

# 29. Gestion des tâches

Le démon possède un ordonnanceur interne.

Chaque opération est représentée par une tâche.

```
Task

↓

Queued

↓

Running

↓

Completed

ou

Cancelled

ou

Failed
```

Toutes les tâches sont identifiées par un UUID.

---

# 30. Priorités

Les tâches possèdent une priorité.

| Priorité | Exemple |
|----------|----------|
| Critical | Fermeture coffre |
| High | Lecture utilisateur |
| Normal | Synchronisation |
| Low | Garbage Collector |
| Background | Nettoyage cache |

L'ordonnanceur peut préempter les tâches de faible priorité.

---

# 31. Concurrence

Chaque moteur est propriétaire de ses données.

Aucun état mutable n'est partagé entre moteurs.

Les échanges se font uniquement :

- par messages ;
- par événements ;
- par interfaces publiques.

Cette règle évite les dépendances circulaires.

---

# 32. Isolation

Le principe fondamental est :

> Celui qui crée une donnée en est le propriétaire.

Exemples :

Storage possède les blocs.

Manifest possède les index.

Vault possède les opérations utilisateur.

Network possède les connexions.

Identity possède les certificats.

Aucun composant ne modifie directement les données d'un autre.

---

# 33. Verrouillage

SyFi privilégie l'architecture orientée messages.

Les verrous sont utilisés uniquement lorsque cela est indispensable.

Ordre de préférence :

1. Ownership
2. Channels
3. Message Passing
4. RwLock
5. Mutex

Les Mutex globaux sont interdits.

---

# 34. Multi-coffres

Le démon peut gérer plusieurs coffres simultanément.

Chaque coffre possède :

- son contexte ;
- son scheduler ;
- son cache ;
- son manifest ;
- son WAL.

Les coffres ne partagent jamais d'état mutable.

```
Daemon

├── Vault A

├── Vault B

├── Vault C

└── Vault D
```

Une erreur sur un coffre ne doit jamais arrêter les autres.

---

# 35. Multi-utilisateurs

L'architecture est conçue pour supporter plusieurs identités.

Chaque identité possède :

- ses clés ;
- ses appareils ;
- ses autorisations.

Les identités sont isolées.

---

# 36. Multi-appareils

Une identité peut enregistrer plusieurs appareils.

Chaque appareil possède :

- un identifiant unique ;
- une paire de clés dédiée ;
- des métadonnées ;
- un état de confiance.

La révocation d'un appareil ne remet pas en cause les autres.

---

# 37. Gestion mémoire

Le Core doit limiter son empreinte mémoire.

Les gros fichiers sont manipulés sous forme de flux.

Les blocs sont chargés uniquement lorsque nécessaire.

Le manifest est partitionné afin d'éviter un chargement intégral.

---

# 38. Gestion des erreurs

Toutes les erreurs sont typées.

Exemple :

```
StorageError

CryptoError

ManifestError

SyncError

NetworkError

IdentityError

IPCError
```

Chaque erreur expose :

- sa catégorie ;
- son origine ;
- un contexte ;
- une possibilité de récupération.

---

# 39. Observabilité

Chaque moteur publie des événements de diagnostic.

Exemples :

```
VaultOpened

ManifestLoaded

GCStarted

PeerConnected

SyncStarted

BlockDownloaded

TransactionCommitted
```

Ces événements alimentent :

- les journaux ;
- l'interface graphique ;
- les métriques ;
- les outils de diagnostic.

---

# 40. Invariants de concurrence

Les règles suivantes sont absolues.

- Un coffre ne peut être fermé pendant une transaction.
- Une transaction ne peut être validée deux fois.
- Un bloc ne peut être collecté tant qu'il est référencé.
- Un manifest ne peut être synchronisé tant qu'il est `Dirty`.
- Deux synchronisations ne peuvent pas modifier simultanément la même partition de manifest.
- Une session réseau compromise est immédiatement isolée.
- Le démon doit rester opérationnel même si un moteur échoue.

Ces invariants devront être vérifiés par des tests d'intégration dédiés.

---

# Conclusion

L'architecture de SyFi repose sur des machines d'état explicites, une forte isolation des responsabilités et une concurrence contrôlée par échange de messages plutôt que par partage d'état.

Cette approche permet d'obtenir un cœur robuste, testable et évolutif, capable de supporter plusieurs coffres, plusieurs identités et plusieurs synchronisations simultanées sans compromettre l'intégrité des données.