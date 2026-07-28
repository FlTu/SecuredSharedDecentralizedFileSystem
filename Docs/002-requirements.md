# docs/002-requirements.md

# Exigences fonctionnelles et non fonctionnelles

Version : 2.0 (Draft)

---

# 1. Objectif

Ce document définit l'ensemble des exigences auxquelles devra satisfaire SyFi.

Chaque exigence possède :

- un identifiant unique
- une priorité
- une description
- un critère d'acceptation

Les exigences sont réparties en deux catégories :

- Fonctionnelles (FR)
- Non fonctionnelles (NFR)

---

# 2. Niveaux de priorité

| Niveau | Signification |
|---------|---------------|
| MUST | Obligatoire pour la V1 |
| SHOULD | Recommandé pour la V1 |
| MAY | Optionnel |
| FUTURE | Prévu après la V1 |

---

# 3. Exigences Fonctionnelles

---

# FR-001 Création d'un coffre

Priorité : MUST

Description

Le système doit permettre la création d'un nouveau coffre sécurisé.

Critères

- génération d'une clé maître
- création de la structure minimale
- création du manifest
- coffre immédiatement exploitable

---

# FR-002 Ouverture d'un coffre

Priorité : MUST

Le système doit permettre l'ouverture d'un coffre existant.

Critères

- dérivation correcte de la clé
- vérification d'intégrité
- chargement des métadonnées

---

# FR-003 Fermeture d'un coffre

Priorité : MUST

La fermeture doit :

- vider toutes les clés mémoire
- fermer les fichiers ouverts
- arrêter les tâches liées au coffre

---

# FR-004 Import d'un fichier

Priorité : MUST

L'utilisateur peut importer un fichier.

Critères

- découpage en blocs
- chiffrement
- stockage
- mise à jour du manifest

---

# FR-005 Lecture d'un fichier

Priorité : MUST

Le système doit restituer exactement les données d'origine.

---

# FR-006 Suppression d'un fichier

Priorité : MUST

La suppression :

- met à jour le manifest
- conserve les tombstones nécessaires
- déclenche le Garbage Collector ultérieurement

---

# FR-007 Déplacement

Priorité : MUST

Le déplacement d'un fichier ne doit pas provoquer sa réécriture.

Seules les métadonnées changent.

---

# FR-008 Renommage

Priorité : MUST

Le renommage ne modifie jamais le contenu.

---

# FR-009 Création de dossier

Priorité : MUST

Le système doit supporter une hiérarchie complète.

---

# FR-010 Suppression de dossier

Priorité : MUST

Suppression récursive.

Conservation des informations nécessaires à la synchronisation.

---

# FR-011 Multi-coffres

Priorité : MUST

Une même instance du démon peut gérer plusieurs coffres simultanément.

---

# FR-012 Synchronisation

Priorité : MUST

Deux appareils doivent pouvoir synchroniser leurs coffres.

---

# FR-013 Synchronisation incrémentale

Priorité : MUST

Seuls les blocs absents doivent être transférés.

---

# FR-014 Détection des conflits

Priorité : MUST

Les modifications concurrentes doivent être détectées.

---

# FR-015 Résolution des conflits

Priorité : SHOULD

Le moteur fournit les informations nécessaires à la résolution.

La résolution automatique est hors périmètre V1.

---

# FR-016 Travail hors ligne

Priorité : MUST

Toutes les opérations locales doivent fonctionner sans connexion.

---

# FR-017 Partage

Priorité : MUST

Le propriétaire peut partager un coffre avec un autre utilisateur.

---

# FR-018 Révocation

Priorité : MUST

Le propriétaire peut retirer un accès.

---

# FR-019 Historique minimal

Priorité : MUST

Le système conserve l'historique nécessaire à la synchronisation.

---

# FR-020 Vérification d'intégrité

Priorité : MUST

Chaque lecture vérifie :

- authentification
- intégrité
- version

---

# FR-021 Reprise après interruption

Priorité : MUST

Après un crash :

- aucune corruption
- reprise automatique

---

# FR-022 Gestion des appareils

Priorité : MUST

Une identité peut posséder plusieurs appareils.

---

# FR-023 Découverte réseau

Priorité : MUST

Découverte automatique sur réseau local.

---

# FR-024 Connexion Internet

Priorité : MUST

Connexion directe lorsque possible.

Sinon :

- relais sécurisé.

---

# FR-025 API publique

Priorité : SHOULD

Toutes les fonctionnalités du Core doivent être accessibles via l'IPC.

---

# 4. Exigences de sécurité

---

# NFR-001 Chiffrement

Priorité : MUST

Toutes les données persistantes sont chiffrées.

---

# NFR-002 Authentification

Priorité : MUST

Toutes les données sont authentifiées.

---

# NFR-003 Confidentialité des métadonnées

Priorité : MUST

Les noms de fichiers ne doivent jamais apparaître en clair sur le disque.

---

# NFR-004 Algorithmes approuvés

Priorité : MUST

Seuls des algorithmes cryptographiques reconnus sont autorisés.

Aucun algorithme "maison".

---

# NFR-005 Rotation des clés

Priorité : SHOULD

Le système doit permettre le renouvellement des clés.

---

# 5. Exigences de performances

---

# NFR-010 Temps d'ouverture

Objectif

< 2 secondes

pour un coffre de taille moyenne.

---

# NFR-011 Taille

Le système doit supporter :

- plusieurs centaines de milliers de fichiers
- plusieurs millions de blocs

---

# NFR-012 Mémoire

Le Core ne doit jamais charger l'intégralité du coffre en mémoire.

---

# NFR-013 Manifest

Le manifest doit être partitionné.

La modification d'un fichier ne doit pas provoquer la réécriture complète du coffre.

---

# NFR-014 Synchronisation

Les téléchargements doivent être parallélisés.

---

# 6. Exigences de résilience

---

# NFR-020 Journal transactionnel

Toutes les opérations critiques passent par un Write-Ahead Log.

---

# NFR-021 Atomicité

Une opération est :

- entièrement validée
- ou entièrement annulée.

Jamais entre les deux.

---

# NFR-022 Crash Recovery

Après un arrêt brutal :

- reprise automatique
- nettoyage
- restauration cohérente

---

# 7. Exigences d'architecture

---

# NFR-030 Modularité

Chaque crate possède une responsabilité unique.

---

# NFR-031 API stable

Les crates communiquent uniquement via leurs interfaces publiques.

---

# NFR-032 Découplage

Le stockage ignore le réseau.

Le réseau ignore le stockage.

Le démon orchestre les deux.

---

# NFR-033 Event Bus

Les composants communiquent via un bus d'événements interne.

Aucun module ne dépend directement d'un autre lorsqu'un échange asynchrone est suffisant.

---

# NFR-034 Versionnement

Tous les formats sont versionnés.

---

# NFR-035 Compatibilité

Une nouvelle version doit pouvoir ouvrir un coffre plus ancien.

---

# 8. Exigences de développement

---

# NFR-040 Rust

Le Core est développé exclusivement en Rust.

---

# NFR-041 Tests

Chaque crate doit disposer :

- de tests unitaires
- de tests d'intégration

---

# NFR-042 Documentation

Toutes les API publiques sont documentées.

---

# NFR-043 CI

Chaque commit déclenche :

- compilation
- lint
- tests

---

# NFR-044 Auditabilité

Chaque décision importante possède un ADR.

---

# 9. Traçabilité

Toutes les futures User Stories, tâches, tests et commits devront référencer les identifiants FR/NFR concernés.

Exemple :

```
Commit :

Implement block cache

Requirements :

FR-004
FR-005
NFR-012
```

Cette traçabilité permettra de mesurer précisément l'avancement du projet.

---

# Conclusion

Les exigences décrites dans ce document constituent le contrat de développement de SyFi.

Toute modification fonctionnelle ou architecturale devra entraîner une mise à jour de ce document avant toute implémentation.