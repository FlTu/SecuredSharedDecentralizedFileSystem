//! ffi — integration Kotlin/Swift/C/C++ (docs/004-workspace.md §17)
//!
//! Aucune logique metier. Traduit l'API du daemon vers une interface C stable,
//! consommee par les clients Android (JNI) et desktop (Tauri).

/// Point de validation minimal du squelette FFI (a remplacer par de vraies
/// fonctions extern "C" une fois le daemon fonctionnel).
#[no_mangle]
pub extern "C" fn syfi_ffi_skeleton_version() -> u32 {
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skeleton_version_is_one() {
        assert_eq!(syfi_ffi_skeleton_version(), 1);
    }
}
