# docs/009-sync.md

# Sync Engine

Version : 2.0 (Draft)

---

# 1. Objectif

Le Sync Engine orchestre la synchronisation d'un coffre entre plusieurs appareils ou pairs.

Il ne stocke aucune donnée, ne chiffre rien, et n'écrit jamais directement dans le Storage (cf. invariant §20 de `003-architecture.md` Partie 2).

Son rôle se limite à :

- comparer deux états de manifest ;
- planifier les transferts ;
- détecter et classifier les conflits (jamais les résoudre lui-même) ;
- reprendre une synchronisation interrompue.

---

# 2. Responsabilités

- comparer Manifest local et Manifest distant ;
- construire une liste d'opérations (téléchargements/uploads) ;
- paralléliser les transferts lorsque possible ;
- détecter les divergences de vecteurs de version ;
- signaler les conflits au Vault (cf. `007-vault.md` §18) ;
- gérer le cycle de vie des tombstones ;
- reprendre automatiquement après interruption.

Le Sync ignore :

- le contenu déchiffré des fichiers ;
- les clés cryptographiques ;
- la structure physique du Storage.

---

# 3. Machine d'état

Cf. `003-architecture.md` Partie 3, §25 :

```
Idle → Discovery → Negotiation → Manifest Exchange → Planning
→ Transfer → Verification → Commit → Completed
```

En cas d'erreur : `Recovery → Retry → Failed`.

---

# 4. Comparaison des manifests (diff Merkle)

1. Les deux pairs échangent la racine de l'arbre de Merkle de chaque partition concernée (cf. `008-manifest.md` §12-15).
2. Si les racines diffèrent, descente dans l'arbre pour isoler précisément les partitions, puis les entrées (`Node`), divergentes.
3. Seules les entrées effectivement différentes sont comparées en détail — pas de comparaison exhaustive fichier par fichier.

Ce mécanisme évite de transmettre ou de comparer l'intégralité du manifest à chaque synchronisation.

---

# 5. Planification et transfert

- Le planificateur construit une file d'opérations (`Download`, `Upload`) à partir du diff.
- Les transferts sont **parallélisés** : plusieurs blocs peuvent être récupérés simultanément, y compris depuis plusieurs pairs si le même coffre est partagé à plusieurs (comportement swarm, façon torrent).
- Chaque bloc reçu suit le même pipeline qu'un import local (`Network → Crypto → Storage → Manifest`, cf. `003-architecture.md` Partie 2 §15 étape 5).

---

# 6. Détection de conflits

Une divergence est un conflit lorsque les vecteurs de version de deux entrées correspondantes sont **incomparables** (ni l'un ni l'autre ne domine l'autre) — cf. `008-manifest.md` pour la structure du vecteur de version.

Le Sync se contente de :

- détecter la divergence ;
- la classifier (type de nœud, taille, extension) ;
- transmettre l'événement `ConflictDetected` au Vault avec les deux versions en cause et leur ancêtre commun si disponible (cf. §8).

Le Sync ne choisit jamais lui-même la résolution — cf. `007-vault.md` §18, qui reste l'autorité de décision.

---

# 7. Politiques de résolution (déléguées au Vault)

Le Vault applique l'une des politiques suivantes selon le type de fichier et la configuration :

1. **Fichiers non-texte** (détection par heuristique de contenu, cf. §8) : conservation des deux versions, renommage de l'une d'elles (`fichier (conflit sur <peer>, <date>).ext`). Aucune perte de donnée silencieuse.
2. **Fichiers texte** : tentative de fusion automatique via le Merge Engine (§8).
3. **Résolution manuelle** : présentée à l'utilisateur via l'application cliente, sur la base des informations fournies par le Sync.
4. **Stratégie personnalisée** (future, hors V1).

---

# 8. Merge Engine — fusion 3-way pour fichiers texte

Fonctionnalité additive par-dessus la politique de base, limitée aux fichiers dont le contenu s'y prête.

## 8.1 Détection du caractère "texte"

Heuristique sur le contenu (UTF-8 valide, absence significative d'octets nuls), pas sur l'extension seule — plus robuste et généralisable à tout fichier texte (code source, markdown, configuration, etc.).

## 8.2 Ancêtre commun

Un vrai 3-way merge nécessite la version ancêtre commune, en plus des deux branches divergentes. Le format du manifest (§9 de `008-manifest.md`) ne conserve que la dernière version par défaut ; en cas de divergence détectée, les blocs de l'ancêtre commun et des deux branches sont **retenus temporairement** (pas un historique complet façon Git — une rétention ciblée, limitée à la durée de la résolution du conflit).

## 8.3 Algorithme

Diff3 ligne à ligne (équivalent à l'algorithme interne de `git merge`) :

- fusion automatique des portions non conflictuelles ;
- insertion de marqueurs de conflit (`<<<<<<<` / `=======` / `>>>>>>>`) pour les portions où les deux branches modifient les mêmes lignes.

## 8.4 Issues

1. **Fusion propre** : application automatique, nouvelle entrée avec un vecteur de version qui domine les deux branches, aucune intervention utilisateur.
2. **Conflit résiduel** : fichier avec marqueurs présenté à l'utilisateur pour édition manuelle ; l'entrée reste marquée `conflit non résolu` jusqu'à validation.

## 8.5 Coût et priorité

Fonctionnalité coûteuse en complexité (gestion de l'ancêtre, UI de résolution dédiée) mais limitée aux fichiers texte détectés — la majorité du volume (fichiers binaires) n'est jamais concernée. Prévue comme itération après un Sync Engine de base stable, pas dans le MVP initial (cf. `015-roadmap.md`).

---

# 9. Tombstones et rétention

Une suppression ne retire jamais immédiatement l'entrée du manifest : le champ `deleted` passe à `true` sur une entrée qui porte elle-même un vecteur de version (cf. `008-manifest.md` §7-8).

- **Durée de rétention par défaut : 1 mois**, personnalisable par coffre.
- Passé ce délai, l'entrée est purgée définitivement de l'index et ne peut plus déclencher de réapparition d'un fichier supprimé.
- Un tombstone "gagne" contre toute version antérieure à la suppression, et "perd" contre une modification réellement postérieure — traité exactement comme un conflit de version ordinaire.

---

# 10. Reprise après interruption

Toutes les opérations de synchronisation sont conçues pour être idempotentes (cf. `003-architecture.md` Partie 2 §18). Une synchronisation interrompue reprend au niveau de la dernière opération validée, sans intervention utilisateur.

---

# 11. Interaction avec le pair hôte

Lorsqu'un pair "hôte" est configuré pour un coffre (cf. `010-network.md`), le Sync tente de s'y connecter en priorité — l'hôte garantissant une disponibilité continue et une réplique intégrale, il constitue un point de rendez-vous stable en complément de la synchronisation directe entre pairs classiques.

---

# 12. Événements

```
SyncStarted
SyncProgress
SyncCompleted
SyncFailed
ConflictDetected
ConflictResolved
TombstonePurged
BlockDownloaded
BlockUploaded
```

---

# 13. Erreurs

```
SyncNegotiationFailed
ManifestExchangeFailed
ConflictUnresolved
TransferFailed
PartitionMismatch
```

Toutes héritent de `SyncError`.

---

# 14. Invariants

- Le Sync n'écrit jamais directement dans le Storage.
- Le Sync ne résout jamais un conflit seul — il détecte, classifie, signale.
- Une entrée supprimée n'est jamais retirée avant l'expiration de sa durée de rétention.
- Deux synchronisations ne modifient jamais simultanément la même partition de manifest (cf. `003-architecture.md` Partie 3 §40).
- Le Merge Engine ne s'applique qu'aux fichiers détectés comme texte ; tout le reste suit la politique de conservation des deux versions.

---

# Conclusion

Le Sync Engine reste un orchestrateur pur : il compare, planifie, transfère et signale, sans jamais devenir une source de vérité ni un exécutant de décisions métier. Cette séparation permet de faire évoluer la politique de résolution de conflits (y compris l'ajout du Merge Engine) sans modifier le protocole de synchronisation lui-même.
