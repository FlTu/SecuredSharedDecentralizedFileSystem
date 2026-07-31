package com.syfi.app

/**
 * Pont vers le crate Rust `ffi` (docs/004-workspace.md §17).
 * Le nom de la bibliotheque ("ffi") doit correspondre au nom du crate
 * (cdylib -> libffi.so), place dans src/main/jniLibs/<abi>/ par cargo-ndk.
 */
object NativeBridge {
    init {
        System.loadLibrary("ffi")
    }

    external fun nativeCreateVault(path: String, passphrase: String): Boolean
    external fun nativeOpenVault(path: String, passphrase: String): Long
    external fun nativeListRoot(handle: Long): String
    external fun nativeCloseVault(handle: Long)
}
