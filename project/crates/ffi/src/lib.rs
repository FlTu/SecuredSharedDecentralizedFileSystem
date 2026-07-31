//! ffi — integration Kotlin/Swift/C/C++ (docs/004-workspace.md §17)
//!
//! Aucune logique metier ici : traduit les appels JNI d'une Activity
//! Android vers l'API du crate `vault`. Le coffre est garde en memoire
//! cote natif via un pointeur brut passe/rendu a Kotlin sous forme de
//! `jlong` (pattern JNI standard pour porter un objet Rust d'un appel a
//! l'autre) — pas de FFI generique bidirectionnelle, uniquement les
//! fonctions necessaires a l'explorateur (docs/015-roadmap.md Phase 9 :
//! explorateur virtuel avant tout montage reel, de toute facon impossible
//! sans root sur Android).

use jni::objects::{JClass, JString};
use jni::sys::{jboolean, jlong, jstring, JNI_FALSE, JNI_TRUE};
use jni::JNIEnv;
use vault::{Vault, ROOT_NODE_ID};

/// Ouvre un coffre et retourne un handle opaque (pointeur Rust encode en
/// jlong) que Kotlin devra repasser aux appels suivants, puis liberer via
/// `nativeCloseVault`. Retourne 0 en cas d'echec (mauvaise passphrase,
/// coffre introuvable, etc.).
///
/// Signature Kotlin correspondante :
/// `external fun nativeOpenVault(path: String, passphrase: String): Long`
#[no_mangle]
pub extern "system" fn Java_com_syfi_app_NativeBridge_nativeOpenVault(
    mut env: JNIEnv,
    _class: JClass,
    path: JString,
    passphrase: JString,
) -> jlong {
    let path: String = match env.get_string(&path) {
        Ok(s) => s.into(),
        Err(_) => return 0,
    };
    let passphrase: String = match env.get_string(&passphrase) {
        Ok(s) => s.into(),
        Err(_) => return 0,
    };

    match Vault::open(&path, passphrase.as_bytes()) {
        Ok(vault) => Box::into_raw(Box::new(vault)) as jlong,
        Err(_) => 0,
    }
}

/// Cree un nouveau coffre (memes signatures/erreurs que ci-dessus).
/// Kotlin : `external fun nativeCreateVault(path: String, passphrase: String): Boolean`
#[no_mangle]
pub extern "system" fn Java_com_syfi_app_NativeBridge_nativeCreateVault(
    mut env: JNIEnv,
    _class: JClass,
    path: JString,
    passphrase: JString,
) -> jboolean {
    let path: String = match env.get_string(&path) { Ok(s) => s.into(), Err(_) => return JNI_FALSE };
    let passphrase: String = match env.get_string(&passphrase) { Ok(s) => s.into(), Err(_) => return JNI_FALSE };

    match Vault::create(&path, passphrase.as_bytes()) {
        Ok(_) => JNI_TRUE,
        Err(_) => JNI_FALSE,
    }
}

/// Liste le contenu de la racine du coffre, serialise en une seule chaine
/// (une entree par ligne : "type\tnom\ttaille\tnode_id_hex") pour rester
/// simple a parser cote Kotlin sans dependance JSON supplementaire dans ce
/// squelette. Kotlin : `external fun nativeListRoot(handle: Long): String`
#[no_mangle]
pub extern "system" fn Java_com_syfi_app_NativeBridge_nativeListRoot(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jstring {
    let vault = unsafe { &*(handle as *const Vault) };
    let mut out = String::new();
    for entry in vault.list_directory(ROOT_NODE_ID) {
        let kind = match entry.entry_type {
            manifest::EntryType::File => "file",
            manifest::EntryType::Directory => "dir",
        };
        let hex: String = entry.node_id.iter().map(|b| format!("{b:02x}")).collect();
        out.push_str(&format!("{kind}\t{}\t{}\t{hex}\n", entry.name, entry.size));
    }
    env.new_string(out).map(|s| s.into_raw()).unwrap_or(std::ptr::null_mut())
}

/// Libere le coffre (equivalent de la fermeture, docs/007-vault.md §9).
/// Kotlin : `external fun nativeCloseVault(handle: Long)`
#[no_mangle]
pub extern "system" fn Java_com_syfi_app_NativeBridge_nativeCloseVault(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    if handle != 0 {
        unsafe { drop(Box::from_raw(handle as *mut Vault)) };
    }
}

#[cfg(test)]
mod tests {
    // Les fonctions extern "system" ne sont testables qu'au travers d'une
    // vraie JVM (docs/004-workspace.md indique que ffi ne contient aucune
    // logique metier a tester en tant que telle) — la logique reelle est
    // deja couverte par les tests du crate `vault`. On verifie seulement
    // ici que le squelette compile en tant que cdylib.
    #[test]
    fn crate_compiles() {
        assert!(true);
    }
}
