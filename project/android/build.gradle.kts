// Racine du projet Android SyFi (docs/004-workspace.md §17 : client JNI du
// crate `ffi`).
//
// Mise a jour du 31/07 (v2) : AGP 9.0+ a completement supprime le support
// du plugin Kotlin separe (org.jetbrains.kotlin.android), pas juste rendu
// optionnel — la premiere tentative de garde-fou (android.builtInKotlin=false)
// s'est revelee elle-meme obsolete/insuffisante avec ce couple de versions.
// Migration complete vers le "Built-in Kotlin" natif d'AGP 9, cf.
// developer.android.com/build/migrate-to-built-in-kotlin et app/build.gradle.kts.
plugins {
    id("com.android.application") version "9.3.0" apply false
}
