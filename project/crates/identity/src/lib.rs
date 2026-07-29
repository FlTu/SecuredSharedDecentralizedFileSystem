//! identity — Identity Engine (docs/011-identity.md)
//!
//! Structure compte/appareil : AccountKey (Ed25519, signe les certificats
//! d'appareil, ne sert jamais de PeerId reseau) + DeviceKey (Ed25519, PeerId
//! reseau reel). Crate independante du reste du Core, reutilisable par
//! d'autres applications de l'ecosysteme.

/// Version du schema d'encodage d'identifiant (docs/011-identity.md §4).
pub const IDENTITY_SCHEMA_VERSION: u8 = 1;

/// Cle publique de compte (place-holder — pas encore de vraie crypto branchee).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountPublicKey(pub Vec<u8>);

/// Cle publique d'appareil, avec certificat brut (signature non verifiee ici).
#[derive(Debug, Clone)]
pub struct DeviceCertificate {
    pub device_public_key: Vec<u8>,
    pub account_public_key: Vec<u8>,
    pub signature: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_version_is_one() {
        assert_eq!(IDENTITY_SCHEMA_VERSION, 1);
    }
}
