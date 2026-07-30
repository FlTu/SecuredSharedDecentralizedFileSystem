# SyFi — squelette de workspace (Phase 1)

Ce squelette a été validé dans un environnement sandbox (Rust 1.75 via apt,
sans rustup) :

- `cargo build --workspace` : succès, 0 warning.
- `cargo test --workspace` : 11 tests passés, 0 échec.
- `cargo run --bin syfi` : exécution correcte.

Il ne contient volontairement **aucune logique métier** — uniquement la
structure de crates de `docs/004-workspace.md` §6, avec un type ou une
fonction minimale par crate pour prouver que les dépendances internes
s'assemblent (ex. `storage`/`manifest`/`vault` dépendent de `common`).

## À faire sur ta machine de développement (pas possible dans ce sandbox)

Ce sandbox n'a pas accès à rustup, Docker, au NDK Android ni à mingw-w64 —
la validation cross-compilée doit se faire chez toi.

1. **Installer rustup** (recommandé plutôt que le paquet `cargo`/`rustc` de
   la distribution, trop ancien pour les dépendances qu'on ajoutera en
   Phase 2, ex. libp2p) :
   ```
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Ajouter les cibles** :
   ```
   rustup target add x86_64-pc-windows-gnu
   rustup target add aarch64-linux-android armv7-linux-androideabi
   ```

3. **Windows** — installer mingw-w64 (`sudo apt install mingw-w64` sur
   Debian/Ubuntu) ou utiliser l'outil `cross` (nécessite Docker) :
   ```
   cargo build --workspace --target x86_64-pc-windows-gnu
   ```

4. **Android** — installer le NDK (via Android Studio ou en standalone) et
   `cargo-ndk` :
   ```
   cargo install cargo-ndk
   cargo ndk -t arm64-v8a -t armeabi-v7a build --workspace
   ```

5. **Clippy et rustfmt** (NFR-029) :
   ```
   rustup component add clippy rustfmt
   cargo clippy --workspace -- -D warnings
   cargo fmt --all -- --check
   ```

## Interface graphique (crate `desktop`)

Le code source d'un explorateur minimal (`crates/desktop/src/main.rs`, basé
sur `eframe`/`egui`) est fourni, mais **n'a pas pu être compilé ni exécuté
dans ce sandbox** : l'arbre de dépendances d'`eframe` (winit, wgpu,
wayland...) dépasse ce que le Rust 1.75 de ce sandbox peut construire, et
le sandbox n'a de toute façon pas de serveur d'affichage pour montrer une
fenêtre. La crate a donc été retirée des membres du workspace (`Cargo.toml`
racine) pour ne pas casser `cargo build --workspace`.

Pour l'essayer chez toi (rustup, Rust récent, environnement graphique) :

1. Rajoute `"crates/desktop"` dans `members` du `Cargo.toml` racine.
2. `cargo run -p desktop`
3. Dans la fenêtre : indique le chemin d'un coffre déjà créé (`syfi create`),
   sa passphrase, clique "Ouvrir", puis sélectionne un fichier et exporte-le.

Comme ce code n'a jamais été compilé, il est possible qu'il reste une ou
deux erreurs de compilation liées à l'API exacte d'`eframe` 0.27 (les
signatures évoluent parfois d'une version mineure à l'autre) — corrige-les
au besoin, ou dis-moi les messages d'erreur et je les corrige avec toi.

## Ce qui est déjà fonctionnel et testé

- `crypto` : 10/10 tests (Argon2id, HKDF, XChaCha20-Poly1305, Ed25519, X25519).
- `storage` : 5/5 tests (Block Store réel sur disque).
- `manifest` : 4/4 tests (index CBOR chiffré, tombstones).
- `vault` : 6/6 tests (create/open/import/export/delete, persistance,
  multi-blocs, rejet de mauvaise passphrase).
- `cli` (`target/debug/syfi`) : testé manuellement en conditions réelles —
  create/import/ls/export bout en bout, avec vérification que le contenu
  physique sur disque ne contient jamais le texte en clair.

## Prochaine étape (Phase 2 restante / Phase 3)

- Ajouter des tests d'intégration multi-fichiers/dossiers imbriqués sur `vault`.
- Une fois la crate `desktop` validée chez toi, on peut enchaîner sur
  `identity` (Phase 4) ou approfondir `manifest` (partitionnement réel,
  Merkle — actuellement une seule table plate).
