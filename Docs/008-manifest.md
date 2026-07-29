# docs/008-manifest.md

# Manifest Engine

## Partie 1 — Fondations

Version : 2.0 (Draft)

---

# 1. Objectif

Le Manifest constitue la représentation logique complète d'un coffre.

Il ne contient jamais les données des fichiers.

Il contient uniquement :

- la structure du coffre ;
- les métadonnées ;
- les références vers les blocs ;
- l'historique logique ;
- les informations nécessaires à la synchronisation.

Le Manifest est la seule source de vérité synchronisée entre tous les appareils.

---

# 2. Responsabilités

Le Manifest Engine est responsable de :

- maintenir l'index logique ;
- suivre les versions ;
- construire les arbres de Merkle ;
- produire les différences ;
- détecter les conflits ;
- fournir les informations nécessaires au Sync Engine.

Le Manifest ne connaît jamais :

- les clés ;
- les blocs ;
- le réseau ;
- les utilisateurs.

---

# 3. Philosophie

Le Manifest est :

- immuable ;
- partitionné ;
- déterministe ;
- sérialisable ;
- reconstruisible.

Il peut être entièrement reconstruit à partir :

- des journaux logiques ;
- des blocs encore référencés.

---

# 4. Pourquoi un Manifest ?

Le Storage répond à une seule question :

> "Donne-moi le bloc X."

Le Vault répond :

> "Déplace ce fichier."

Le Manifest répond :

> "Quels blocs composent ce fichier, où se trouve-t-il et quelle est sa version ?"

---

# 5. Structure générale

Le Manifest est constitué de plusieurs composants.

```
Manifest

├── Header

├── Partitions

├── Journal

├── Merkle Index

├── Version Vectors

└── Footer
```

Chaque composant possède une responsabilité unique.

---

# 6. Header

Le Header contient :

```
Manifest Version

Vault ID

Creation Time

Last Update

Partition Count

Format Version

Flags
```

Le Header est volontairement réduit.

---

# 7. Journal logique

Toutes les modifications sont enregistrées sous forme d'événements immuables.

Exemple :

```
CreateNode

MoveNode

RenameNode

DeleteNode

UpdateMetadata

CreateDirectory

RestoreVersion
```

Le journal est append-only.

Aucune entrée n'est modifiée.

---

# 8. Les Nodes

Chaque élément du coffre est représenté par un Node.

```
NodeId

↓

Metadata

↓

Block References

↓

Children (si dossier)
```

Le Node ne contient jamais directement les données.

---

# 9. Références vers les blocs

Chaque fichier possède une liste ordonnée.

```
File

↓

Block A

↓

Block B

↓

Block C
```

Le Manifest ne connaît que les identifiants des blocs.

---

# 10. Arborescence

Les répertoires sont représentés sous forme d'arbre.

```
Root

├── Documents

│   ├── Rapport.pdf

│   └── Budget.xlsx

└── Photos

    └── Vacances.jpg
```

Chaque relation est définie par les `NodeId`.

---

# 11. Métadonnées

Le Manifest synchronise uniquement les métadonnées globales.

Exemple :

- nom ;
- taille logique ;
- auteur ;
- permissions ;
- dates ;
- version.

Les informations purement locales sont exclues.

---

# 12. Partitionnement

Le Manifest n'est plus un fichier monolithique.

Il est découpé en partitions indépendantes.

```
Manifest

├── Partition A

├── Partition B

├── Partition C

└── Partition D
```

Chaque partition peut être synchronisée indépendamment.

---

# 13. Pourquoi partitionner ?

Sans partitionnement :

```
1 modification

↓

Manifest complet modifié
```

Avec partitionnement :

```
1 modification

↓

1 seule partition modifiée
```

Les échanges réseau sont fortement réduits.

---

# 14. Critères de partition

Une partition peut être construite selon :

- un sous-arbre logique ;
- une plage de `NodeId` ;
- un espace de noms.

L'algorithme exact est laissé à l'implémentation tant que les partitions restent stables dans le temps.

---

# 15. Identité d'une partition

Chaque partition possède :

```
PartitionId

Version Vector

Merkle Root

Node Count

Checksum
```

Elle peut être vérifiée indépendamment des autres.

---

# 16. Immutabilité

Une partition validée n'est jamais modifiée en place.

Toute évolution produit une nouvelle révision logique.

Cela facilite :

- les comparaisons ;
- les restaurations ;
- les synchronisations.

---

# 17. Événements

Le Manifest publie notamment :

```
PartitionUpdated

NodeCreated

NodeDeleted

NodeMoved

ManifestCommitted

ManifestLoaded
```

Ces événements sont consommés par le Sync Engine et le Daemon.

---

# 18. Erreurs

Principales erreurs :

```
ManifestCorrupted

InvalidPartition

UnknownNode

InvalidReference

VersionConflict

UnsupportedFormat
```

Toutes héritent de `ManifestError`.

---

# 19. Invariants

Les règles suivantes sont absolues.

- Chaque Node appartient à une seule partition.
- Chaque référence de bloc doit exister dans le Storage.
- Une partition possède toujours un `PartitionId` unique.
- Le Manifest ne contient jamais de données utilisateur.
- Les écritures sont append-only.
- Une partition ne peut être validée qu'après une transaction complète.

---

# Conclusion

Le Manifest constitue l'index distribué du coffre.

En séparant les métadonnées des données physiques et en introduisant un partitionnement natif, SyFi peut faire évoluer des coffres contenant plusieurs millions de fichiers sans devoir recalculer ni transmettre l'intégralité de leur structure à chaque modification.