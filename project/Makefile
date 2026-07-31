# Makefile racine du superprojet SyFi (docs/004-workspace.md).
#
# A utiliser sur une machine avec l'environnement complet (rustup, cibles
# Android, NDK, cargo-ndk, JDK 17, Android SDK) — ce sandbox de
# developpement n'a rien de tout cela (cf. README.md, section Android),
# c'est pourquoi .github/workflows/build.yml existe en complement : il fait
# le meme travail sur les runners GitHub, avec acces reseau complet.

.PHONY: build test build-android clean

## Compile tout le workspace Rust natif (Linux).
build:
	cargo build --workspace

## Lance tous les tests du workspace Rust natif.
test:
	cargo test --workspace

## Compile la lib native pour Android (arm64-v8a + armeabi-v7a) puis l'APK
## debug. Necessite : rustup target add aarch64-linux-android
## armv7-linux-androideabi ; cargo install cargo-ndk ; un NDK installe et
## ANDROID_NDK_HOME positionne ; un SDK Android avec les build-tools.
build-android:
	cargo ndk -t arm64-v8a -t armeabi-v7a \
		-o android/app/src/main/jniLibs \
		build -p ffi --release
	cd android && ./gradlew assembleDebug

## Nettoie les artefacts de build Rust et Android.
clean:
	cargo clean
	cd android && ./gradlew clean || true
