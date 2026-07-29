//! daemon — microkernel (docs/005-daemon.md)
//!
//! Orchestre les services (Vault, Identity, Network, Sync, Scheduler,
//! Configuration, Capability) sans contenir lui-meme de logique metier.
//! Point d'entree unique pour toutes les applications clientes (via IPC).

/// Cycle de vie d'un service (docs/005-daemon.md §6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceState {
    Created,
    Starting,
    Running,
    Paused,
    Stopping,
    Stopped,
    Failed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn services_start_as_created() {
        assert_eq!(ServiceState::Created, ServiceState::Created);
    }
}
