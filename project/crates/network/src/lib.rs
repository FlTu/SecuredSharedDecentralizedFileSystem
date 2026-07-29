//! network — Network Engine (docs/010-network.md)
//!
//! Transport, decouverte, sessions, relais, NAT traversal. Ignore
//! totalement le contenu des fichiers et le manifest en clair.
//!
//! TODO (hors squelette) : integration libp2p (mDNS, Kademlia DHT, Noise,
//! relay+hole punching) une fois la crate `identity` stabilisee.

/// Etats d'une session reseau (docs/003-architecture.md Partie 3 §26).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    Disconnected,
    Connecting,
    Authenticated,
    Negotiating,
    Ready,
    Streaming,
    Closing,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_starts_disconnected() {
        let s = SessionState::Disconnected;
        assert_eq!(s, SessionState::Disconnected);
    }
}
