# docs/000-introduction.md

# SyFi
## Système de Fichiers Décentralisé, Chiffré et Synchronisé

**Version : 2.0 (Draft)**

Auteur : Projet SyFi

Statut : Spécification officielle

---

# Préface

SyFi est un système de stockage distribué conçu pour offrir à un utilisateur la maîtrise complète de ses données sans dépendre d'un fournisseur de cloud.

Le projet est né d'un constat simple :

Aujourd'hui, la majorité des solutions de synchronisation de fichiers reposent sur un serveur central appartenant à un tiers. Même lorsque les données sont chiffrées, les métadonnées restent généralement visibles : noms de fichiers, structure des répertoires, historique de synchronisation, identité des utilisateurs, fréquence des accès, taille des fichiers, etc.

SyFi poursuit une approche différente.

Le réseau n'est plus considéré comme un espace de confiance.

Chaque pair est potentiellement hostile.

Chaque transfert est considéré comme observable.

Chaque stockage distant est considéré comme compromis.

La sécurité du système repose uniquement sur les primitives cryptographiques, jamais sur l'infrastructure.

---

# Philosophie

Le projet repose sur plusieurs principes fondateurs.

## 1. Le propriétaire est l'utilisateur

Aucune entreprise ne possède les données.

Aucun serveur n'est indispensable.

Aucun abonnement n'est nécessaire.

Le réseau n'est qu'un moyen de transport.

Le propriétaire des données reste leur détenteur.

---

## 2. Le chiffrement n'est pas une option

Toutes les données sont chiffrées avant d'être écrites sur le disque.

Toutes les données sont chiffrées avant d'être transmises.

Toutes les métadonnées sont protégées.

Le stockage physique ne révèle jamais :

- les noms de fichiers
- la hiérarchie des dossiers
- le type des fichiers
- leur contenu

---

## 3. L'ouverture est une exigence

Le format du coffre est public.

La documentation est publique.

Les algorithmes sont publics.

Aucun composant critique ne dépend d'un service propriétaire.

Un utilisateur doit pouvoir récupérer ses données uniquement à partir de :

- la documentation officielle
- sa phrase de passe
- une implémentation minimale indépendante

L'application n'est jamais indispensable pour retrouver ses fichiers.

---

## 4. Une seule implémentation du cœur

Le cœur applicatif est entièrement développé en Rust.

Aucune logique métier ne doit être réécrite dans les interfaces.

Les interfaces (Linux, Windows, Android et futures plateformes) sont uniquement des clientes du Core.

Cette approche garantit :

- un comportement identique sur toutes les plateformes ;
- une réduction des bugs ;
- une surface d'audit plus faible ;
- une maintenance simplifiée.

---

## 5. Le réseau est interchangeable

Le stockage ne dépend pas du réseau.

Le réseau ne dépend pas du stockage.

Les moteurs de synchronisation, de chiffrement et de transport sont indépendants.

Il doit être possible de remplacer un protocole réseau sans modifier le moteur du coffre.

Cette indépendance constitue une exigence architecturale.

---

## 6. Les applications ne possèdent pas les données

Le partage de fichiers n'est que l'une des applications de l'écosystème.

Le cœur du système est un démon local chargé de gérer :

- les identités ;
- les coffres ;
- les synchronisations ;
- les connexions réseau ;
- les politiques de sécurité.

Les interfaces graphiques deviennent des clientes du démon.

Cette architecture permet à plusieurs applications de partager le même moteur sans duplication de code.

---

# Objectifs

La première version de SyFi doit permettre :

- créer un coffre chiffré ;
- synchroniser ce coffre entre plusieurs appareils ;
- partager un coffre avec un autre utilisateur ;
- fonctionner sans serveur central ;
- préserver la confidentialité des métadonnées ;
- fonctionner hors ligne ;
- détecter automatiquement les modifications ;
- transférer uniquement les blocs nécessaires ;
- compiler depuis Linux vers Linux, Windows et Android.

---

# Hors périmètre de la Version 1

Les fonctionnalités suivantes sont volontairement reportées :

- synchronisation différentielle par contenu (content-defined chunking) ;
- déduplication globale ;
- collaboration temps réel ;
- édition simultanée ;
- groupes de grande taille ;
- historique complet façon Git ;
- iOS ;
- interface Web ;
- plugins ;
- scripts utilisateurs.

Leur absence ne doit toutefois pas empêcher leur intégration ultérieure.

L'architecture devra donc rester extensible.

---

# Principes d'architecture

Afin de garantir cette évolutivité, plusieurs règles sont imposées.

## Modularité

Chaque moteur possède une responsabilité unique.

Aucun module ne doit connaître les détails internes d'un autre.

Les communications se font uniquement via des interfaces publiques.

---

## Compatibilité

Tous les formats utilisés par SyFi sont versionnés.

Une nouvelle version du logiciel doit pouvoir ouvrir un coffre créé plusieurs années auparavant.

La compatibilité ascendante constitue un objectif majeur.

---

## Auditabilité

Chaque primitive cryptographique est documentée.

Chaque format binaire possède une spécification.

Chaque protocole réseau est décrit indépendamment du code.

Le comportement du logiciel doit pouvoir être vérifié sans lecture du code source.

---

## Résilience

Une panne électrique.

Une fermeture brutale.

Une perte réseau.

Un crash.

Aucun de ces événements ne doit pouvoir corrompre un coffre.

Toutes les opérations critiques sont transactionnelles.

---

## Évolutivité

L'architecture est pensée pour plusieurs dizaines de millions de blocs.

Plusieurs centaines de milliers de fichiers.

Plusieurs appareils.

Plusieurs coffres.

Plusieurs applications utilisant simultanément le même démon.

---

# Vision long terme

SyFi n'a pas vocation à devenir uniquement une application de synchronisation.

Le projet constitue la première brique d'un écosystème plus vaste.

À terme, plusieurs applications partageront les mêmes fondations :

- partage de fichiers ;
- messagerie chiffrée ;
- synchronisation de données ;
- sauvegarde distribuée ;
- partage sécurisé entre appareils.

Toutes utiliseront :

- le même système d'identité ;
- le même démon ;
- la même couche réseau ;
- les mêmes primitives cryptographiques.

Cette convergence permettra de limiter la duplication de code et de garantir une cohérence fonctionnelle entre les applications.

---

# Portée de cette spécification

Ce document décrit exclusivement l'architecture de référence de SyFi.

Il ne constitue pas une documentation utilisateur.

Il ne décrit pas l'interface graphique.

Il définit :

- les contraintes techniques ;
- les contrats entre modules ;
- les protocoles ;
- les formats ;
- les décisions d'architecture.

Les documents suivants détaillent chacun des sous-systèmes.

Aucun développement ne doit s'écarter de cette spécification sans la création préalable d'un ADR (Architecture Decision Record).

---

**Document suivant :**

`docs/001-vision.md`