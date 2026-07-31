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

## Android (APK)

**Non compilable dans ce sandbox** : pas de SDK Android, pas de Gradle, pas
de NDK disponibles ici (réseau restreint aux domaines listés dans la
configuration du sandbox, qui n'incluent pas les serveurs de Google/Android).
Tout ce qui suit a été écrit avec soin mais **jamais construit ni exécuté** —
à valider chez toi comme pour la crate `desktop`.

### Ce qui est fourni

- `crates/ffi/src/lib.rs` : pont JNI réel vers `vault` (compile dans ce
  sandbox — `cargo build -p ffi` passe). Quatre fonctions : créer, ouvrir,
  lister la racine, fermer un coffre. Signatures Kotlin correspondantes en
  commentaire au-dessus de chaque fonction Rust.
- `android/` : projet Gradle/Kotlin minimal — une seule Activity, deux
  champs (chemin, passphrase), deux boutons (créer, ouvrir), une liste
  affichant le contenu de la racine du coffre. Pas de montage système réel
  (impossible sans root sur Android, cf. `015-roadmap.md` §5) — c'est
  l'explorateur virtuel interne, comme prévu.

### Étapes pour obtenir un APK, chez toi

1. **Installer le NDK** (via Android Studio → SDK Manager → SDK Tools →
   NDK, ou en standalone) et `cargo-ndk` :
   ```
   cargo install cargo-ndk
   rustup target add aarch64-linux-android armv7-linux-androideabi
   ```

2. **Compiler le crate `ffi` pour Android** — depuis la racine du workspace
   Rust (`project/`, pas `project/android/`) :
   ```
   cargo ndk -t arm64-v8a -t armeabi-v7a -o android/app/src/main/jniLibs build -p ffi --release
   ```
   Ça doit produire `android/app/src/main/jniLibs/arm64-v8a/libffi.so` et
   l'équivalent `armeabi-v7a/`.

3. **Ouvrir `android/` dans Android Studio** (ou `cd android && ./gradlew
   assembleDebug` en ligne de commande une fois le wrapper Gradle généré —
   `gradle wrapper` si tu as Gradle installé globalement, sinon Android
   Studio le fait automatiquement à l'ouverture du projet).

4. **Lancer sur un appareil/émulateur** — le champ "chemin du coffre"
   attend un chemin accessible à l'app (`/data/data/com.syfi.app/files/...`
   ou un chemin sur le stockage partagé avec la permission adéquate, à
   ajouter dans `AndroidManifest.xml` si tu testes sur stockage partagé).

### Limites connues de ce squelette Android

- Pas de gestion de permissions runtime (stockage) — à ajouter avant tout
  test sur un chemin hors du répertoire privé de l'app.
- `nativeListRoot` sérialise en texte tabulé plutôt qu'en JSON pour éviter
  une dépendance supplémentaire à ce stade — à revoir si le format doit
  transporter plus que type/nom/taille/id.
- Aucune gestion d'erreur fine côté Kotlin au-delà d'un message générique —
  suffisant pour valider que la chaîne complète fonctionne, pas pour un
  usage réel.

## Interface graphique desktop (crate `desktop`)

## Interface graphique desktop (crate `desktop`)

Le code source d'un explorateur minimal (`crates/desktop/src/main.rs`, basé
sur `eframe`/`egui`) est fourni. **Bug corrige le 31/07** : la signature du
closure passe a `eframe::run_native` a ete adaptee a l'API d'`eframe` 0.27
(`Box<dyn App>` attendu directement, pas `Result<Box<dyn App>, _>` — cette
derniere forme n'existe qu'a partir d'une version ulterieure d'eframe).
Toujours **non compile dans ce sandbox** pour les memes raisons que
precedemment (toolchain, pas de serveur d'affichage) — a valider chez toi
en premier avec `cargo run -p desktop` apres avoir remis `"crates/desktop"`
dans les membres du workspace racine.

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
