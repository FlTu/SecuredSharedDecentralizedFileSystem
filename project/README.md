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

## Prochaine étape (Phase 2 de `015-roadmap.md`)

Une fois les trois cibles validées chez toi, on attaque les crates
`crypto`, `storage` et `vault` avec une vraie logique métier (docs/006,
007, 013).
