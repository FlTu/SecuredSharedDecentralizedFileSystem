// Migration vers le "Built-in Kotlin" d'AGP 9.0+ (31/07) :
// - plus de plugin org.jetbrains.kotlin.android (supprime, pas juste
//   optionnel, a partir d'AGP 9.0 — cf. developer.android.com/build/migrate-to-built-in-kotlin)
// - la config du compilateur Kotlin passe par un bloc `kotlin {}` de haut
//   niveau plutot que par `android { kotlinOptions {} }`
plugins {
    id("com.android.application")
}

android {
    namespace = "com.syfi.app"
    // Aligne sur la plateforme deja presente dans le SDK partage (root-owned,
    // /usr/lib/android-sdk) — evite d'avoir a installer platforms;android-34
    // avec sudo. AGP 9.3 supporte jusqu'a l'API 37 (36 est donc dans la marge).
    compileSdk = 36

    defaultConfig {
        applicationId = "com.syfi.app"
        minSdk = 26
        targetSdk = 34
        versionCode = 1
        versionName = "0.1-squelette"
    }

    buildTypes {
        release {
            isMinifyEnabled = false
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    // Les .so natifs (produits par cargo-ndk depuis crates/ffi) sont
    // attendus dans src/main/jniLibs/<abi>/libffi.so — pas de configuration
    // supplementaire necessaire, c'est l'emplacement standard reconnu par
    // le plugin Android.
}

kotlin {
    compilerOptions {
        jvmTarget = org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_17
    }
}

dependencies {
    implementation("androidx.core:core-ktx:1.13.1")
    implementation("androidx.appcompat:appcompat:1.7.0")
}
