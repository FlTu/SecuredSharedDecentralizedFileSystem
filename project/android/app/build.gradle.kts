plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
}

android {
    namespace = "com.syfi.app"
    compileSdk = 34

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
    kotlinOptions {
        jvmTarget = "17"
    }

    // Les .so natifs (produits par cargo-ndk depuis crates/ffi) sont
    // attendus dans src/main/jniLibs/<abi>/libffi.so — pas de configuration
    // supplementaire necessaire, c'est l'emplacement standard reconnu par
    // le plugin Android.
}

dependencies {
    implementation("androidx.core:core-ktx:1.13.1")
    implementation("androidx.appcompat:appcompat:1.7.0")
}
