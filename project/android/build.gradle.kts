// Racine du projet Android SyFi (docs/004-workspace.md §17 : client JNI du
// crate `ffi`). Versions de plugin a ajuster selon ta version d'Android
// Studio installee — celles-ci sont un point de depart raisonnable, non
// verifie faute d'environnement Android dans le sandbox de developpement.
plugins {
    id("com.android.application") version "8.5.0" apply false
    id("org.jetbrains.kotlin.android") version "1.9.24" apply false
}
