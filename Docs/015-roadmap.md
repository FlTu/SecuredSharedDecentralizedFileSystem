# docs/015-roadmap.md

# Feuille de route

Version : 2.0 (Draft)

---

# 1. Objectif

Ce document définit l'ordre de développement recommandé, cohérent avec les principes de modularité de `003-architecture.md` : chaque phase doit être testable indépendamment avant que la suivante ne s'appuie dessus.

---

# 2. Phases

## Phase 1 — Squelette workspace et validation cross-compilation

- Structure de workspace complète (`004-workspace.md` §6) : toutes les crates présentes dès le départ, même en stub (`common`, `crypto`, `storage`, `manifest`, `vault`, `sync`, `network`, `identity`, `daemon`, `ipc`, `ffi`, `cli`).
- Validation de compilation native Linux, puis croisée Windows (`cross`/mingw-w64) et Android (`cargo-ndk`) **avant tout développement métier substantiel** — un problème de toolchain découvert tard coûte beaucoup plus cher qu'un problème découvert sur un squelette vide.
- Aucune logique métier à ce stade, uniquement de quoi prouver que chaque crate compile et s'exécute sur les trois cibles.

## Phase 2 — Fondations locales (Crypto, Storage, Vault)

- Crate `crypto` : primitives de base (`013-security.md` §3), dérivation de clés, tests unitaires.
- Crate `storage` : blocs de taille fixe, UUID opaque, WAL, cycle écriture/lecture, séparation Block Store / Local Index Store (`006-storage.md`).
- Crate `vault` (sous-ensemble minimal) : création/ouverture/fermeture de coffre, import/export simple.
- CLI minimale pour valider le format sur disque sans dépendre d'aucune interface graphique.
- Aucun réseau à ce stade.

## Phase 3 — Manifest

- Crate `manifest` : index partitionné, vecteurs de version, Merkle (`008-manifest.md`).
- Toujours en local — validation du cycle complet import → manifest → lecture sans réseau.

## Phase 4 — Identity

- Structure compte/appareil, certificats, encodage d'identifiant (`011-identity.md`).
- Toujours sans connexion réseau réelle — génération et vérification locales uniquement.

## Phase 5 — Network (découverte)

- Découverte LAN (mDNS) en premier, DHT ensuite (`010-network.md` §2).
- Sessions réseau de base, sans encore les sessions de manifest (§6 du même document).

## Phase 6 — Sync (protocole de base)

- Diff Merkle, planification, transfert de blocs entre deux pairs sur LAN.
- Politique de conflit simple uniquement (conservation des deux versions) — pas encore le Merge Engine.

## Phase 7 — Rôle de pair hôte et sessions de manifest

- Une fois le P2P direct stable entre deux pairs classiques.
- Sessions de manifest avec fermeture pilotée par le cycle de vie applicatif (`010-network.md` §6).

## Phase 8 — Merge Engine

- Fusion 3-way pour fichiers texte (`009-sync.md` §8) — itération après un Sync Engine stable, pas dans le MVP.

## Phase 9 — Interfaces

- Explorateur virtuel interne d'abord (accès aux fichiers via l'API du Vault, sans montage système) — c'est de toute façon la seule option sur Android (pas de FUSE sans root).
- Montage système réel (FUSE via `fuser` sur Linux, WinFsp sur Windows) ajouté **seulement une fois le moteur validé** par l'explorateur interne — pas en parallèle. FUSE/WinFsp sont des dépendances externes non triviales ; les introduire avant que Vault/Manifest/Sync soient stables complique inutilement le débogage.
- Desktop (Tauri ou egui), Android (Kotlin + JNI), CLI.
- Le Core doit être fonctionnellement complet avant cette phase — les interfaces ne sont que des clientes.

## Phase 10 — Écosystème

- Finalisation du protocole IPC et des scopes de capacité (`012-ipc.md`).
- Composants d'UI partagés, une fois les besoins réels des autres applications (messagerie notamment) mieux connus.

---

# 3. Jalons de validation cross-compilation

1. Compilation native Linux réussie (sanity check du workspace).
2. Compilation croisée Windows (`cross` ou mingw-w64) sans modification du code métier.
3. Compilation croisée Android (`cargo-ndk` + NDK) sans modification du code métier.
4. Un même jeu de tests unitaires passe sur les trois cibles.

Toute divergence de comportement entre cibles à ce stade doit être traitée comme un défaut d'architecture (dépendance non isolée derrière un adaptateur, cf. `003-architecture.md` Partie 4 §42), pas comme un correctif ponctuel.

---

# 4. Critères de sortie de la V1

Repris de `001-vision.md` §9 :

- un coffre peut être créé ;
- plusieurs appareils peuvent être synchronisés sans serveur central ;
- un coffre peut être partagé ;
- les données restent lisibles indépendamment de l'application (outil de référence, `014-file-format.md` §8) ;
- compilation réussie vers Linux, Windows et Android depuis un poste Linux.

---

# 5. Risques connus

- **Maturité de libp2p en Rust** pour certains mécanismes de relais/hole punching — à valider tôt (Phase 5) plutôt qu'en fin de projet.
- **Montage virtuel** : FUSE (Linux, via `fuser`) et WinFsp (Windows) sont des dépendances externes à l'utilisateur final (installation séparée) ; sur Android, pas de montage réel possible sans root — l'explorateur intégré (Phase 9) est la seule option réaliste, pas un palliatif temporaire.
- **Toolchain NDK** : versions à figer explicitement dans la documentation de build, la compatibilité `cargo-ndk`/NDK évoluant vite.

---

# 6. Points ouverts à lever avant implémentation (renvoi)

- Durée de validité et renouvellement du certificat d'appareil — `011-identity.md` §5.
- Liste exacte des portées de capacité IPC au-delà du socle minimal — `012-ipc.md` §4.2.
- Valeur exacte du timeout de secours des sessions de manifest — `010-network.md` §6.2.

Ces points sont volontairement non bloquants pour démarrer la Phase 1.

---

# Conclusion

Cette feuille de route privilégie la validation du cœur (crypto, stockage, manifest) et de la portabilité cross-compilée avant tout investissement dans le réseau ou l'interface — conformément au principe de `003-architecture.md` selon lequel aucune couche ne doit dépendre d'une couche supérieure encore instable.
