//! sync — Sync Engine (docs/009-sync.md)
//!
//! Compare deux manifests, planifie les transferts, detecte et signale les
//! conflits (ne les resout jamais lui-meme — delegue au Vault). N'ecrit
//! jamais directement dans le Storage.

/// Resultat de comparaison de deux entrees d'index (docs/009-sync.md §6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffOutcome {
    Identical,
    LocalIsNewer,
    RemoteIsNewer,
    Conflict,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_is_not_a_conflict() {
        assert_ne!(DiffOutcome::Identical, DiffOutcome::Conflict);
    }
}
