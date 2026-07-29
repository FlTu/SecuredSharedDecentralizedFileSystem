# docs/007-vault.md

# Vault Engine

Version : 2.0 (Draft)

---

# 1. Objectif

Le Vault Engine constitue le cœur métier de SyFi.

Il représente un coffre logique contenant :

- des fichiers ;
- des dossiers ;
- des liens de partage ;
- des métadonnées ;
- les opérations utilisateur.

Le Vault est le seul composant autorisé à manipuler ces concepts.

---

# 2. Responsabilités

Le Vault est responsable de :

- créer un coffre ;
- ouvrir un coffre ;
- fermer un coffre ;
- importer des fichiers ;
- exporter des fichiers ;
- déplacer des éléments ;
- renommer des éléments ;
- supprimer des éléments ;
- partager des éléments ;
- restaurer des versions.

Il délègue :

- le chiffrement au Crypto Engine ;
- le stockage au Storage Engine ;
- l'indexation au Manifest Engine ;
- la synchronisation au Sync Engine.

---

# 3. Principes

Le Vault applique les principes suivants.

## Source de vérité métier

Le Vault est propriétaire de la structure logique.

Ni le Storage ni le Manifest ne prennent de décisions métier.

---

## Transactions

Chaque modification est atomique.

Une opération est soit entièrement appliquée, soit entièrement annulée.

---

## Immutabilité

Une modification ne remplace jamais un contenu existant.

Toute modification produit une nouvelle version logique.

---

# 4. Structure logique

Un coffre contient exclusivement des nœuds.

```
Vault

└── Node
```

Les nœuds peuvent être :

```
File

Directory

Symlink (future)

Shared Link

Special Node
```

---

# 5. Identifiants

Chaque nœud possède un identifiant immuable.

```
NodeId (UUID v7)

ParentId

CreatedAt

UpdatedAt
```

Le nom d'un fichier n'est jamais utilisé comme identifiant.

---

# 6. Métadonnées

Chaque nœud possède :

```
Nom

Type

Taille logique

Date de création

Date de modification

Auteur

Version

Permissions

Tags (future)

Favori (local uniquement)
```

Les métadonnées locales ne sont jamais synchronisées.

---

# 7. Création d'un coffre

Pipeline :

```
Utilisateur

↓

Daemon

↓

Vault

↓

Crypto

↓

Storage

↓

Manifest

↓

Validation
```

À la création :

- génération de l'identité du coffre ;
- création de la clé maîtresse ;
- initialisation du Manifest ;
- création de la structure racine.

---

# 8. Ouverture d'un coffre

Le Vault :

- vérifie le format ;
- déverrouille les clés ;
- charge le Manifest ;
- initialise les caches ;
- publie `VaultOpened`.

Le Storage n'est chargé qu'à la demande.

---

# 9. Fermeture

Avant fermeture :

- terminer les transactions ;
- arrêter les synchronisations actives ;
- vider les caches ;
- nettoyer les secrets en mémoire ;
- publier `VaultClosed`.

---

# 10. Import d'un fichier

Pipeline :

```
Lecture

↓

Chunking

↓

Crypto

↓

Storage

↓

Manifest

↓

Commit
```

Le Vault orchestre chaque étape.

---

# 11. Export

Pipeline :

```
Manifest

↓

Storage

↓

Crypto

↓

Assemblage

↓

Écriture
```

Le Vault reconstitue le flux d'origine.

---

# 12. Déplacement

Déplacer un fichier modifie uniquement :

```
ParentId

Chemin logique
```

Aucun bloc n'est réécrit.

---

# 13. Renommage

Le renommage modifie uniquement :

```
Nom

Date de modification

Version
```

Le contenu reste inchangé.

---

# 14. Suppression

Une suppression crée un marqueur logique ("tombstone").

Les blocs restent présents jusqu'au passage du Garbage Collector.

Cette stratégie garantit une synchronisation fiable entre appareils.

---

# 15. Restauration

Une version supprimée peut être restaurée tant que les blocs existent encore.

La restauration crée une nouvelle version logique.

---

# 16. Historique

Chaque modification produit un nouvel état.

```
Version 1

↓

Version 2

↓

Version 3
```

Le mécanisme de conservation est défini par une politique configurable.

---

# 17. Verrouillage

Le Vault protège les opérations concurrentes.

Deux opérations incompatibles sur un même nœud ne peuvent être validées simultanément.

Les lectures restent autorisées.

---

# 18. Gestion des conflits

Le Vault reçoit les conflits détectés par le Sync Engine.

Il applique la politique configurée :

- Last Writer Wins ;
- conservation des deux versions ;
- résolution manuelle ;
- stratégie personnalisée (future).

---

# 19. Recherche

Le Vault expose une API de recherche.

Critères possibles :

- nom ;
- extension ;
- taille ;
- date ;
- auteur ;
- type.

Les futurs index plein texte utiliseront un moteur séparé.

---

# 20. Partage

Le Vault crée des objets de partage.

Ils référencent :

- un ou plusieurs nœuds ;
- des permissions ;
- une durée de validité.

Le contenu n'est jamais dupliqué.

---

# 21. Permissions

Chaque opération vérifie les droits associés.

Exemples :

```
Read

Write

Delete

Share

Admin
```

Le Vault applique ces règles avant toute modification.

---

# 22. Cache métier

Le Vault maintient un cache léger :

- arborescence récemment consultée ;
- chemins résolus ;
- métadonnées fréquentes.

Les contenus des fichiers ne sont jamais conservés durablement.

---

# 23. Événements

Le Vault publie notamment :

```
VaultOpened

VaultClosed

FileImported

FileExported

NodeMoved

NodeRenamed

NodeDeleted

ConflictDetected

VersionRestored
```

Les consommateurs s'abonnent via l'Event Bus.

---

# 24. Erreurs

Principales erreurs :

```
VaultLocked

NodeNotFound

InvalidPath

PermissionDenied

ConflictDetected

UnsupportedVersion

TransactionFailed
```

Toutes héritent de `VaultError`.

---

# 25. Invariants

Les règles suivantes sont absolues.

- Chaque nœud possède un identifiant unique.
- Le répertoire racine ne peut être supprimé.
- Un déplacement ne modifie jamais le contenu d'un fichier.
- Un renommage ne modifie jamais les blocs.
- Toute modification passe par une transaction.
- Le Vault ne chiffre jamais directement les données.
- Le Vault n'écrit jamais directement sur le disque.

---

# 26. Interfaces publiques

Le Vault expose notamment :

```
CreateVault

OpenVault

CloseVault

ImportFile

ExportFile

MoveNode

RenameNode

DeleteNode

RestoreVersion

ListDirectory

Search
```

Ces opérations constituent l'API métier officielle.

---

# Conclusion

Le Vault Engine est le cœur fonctionnel de SyFi.

Il garantit la cohérence de l'espace de travail utilisateur tout en déléguant les responsabilités techniques aux autres moteurs. Cette séparation permet de faire évoluer indépendamment le stockage, la synchronisation ou la cryptographie sans remettre en cause les règles métier.