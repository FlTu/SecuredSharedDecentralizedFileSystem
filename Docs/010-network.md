# docs/010-network.md

# Network Engine

Version : 2.0 (Draft)

---

# 1. Objectif

Le Network Engine transporte des messages entre pairs. Il ignore totalement le contenu des fichiers, le chiffrement des blocs et le manifest (cf. `003-architecture.md` Partie 2, §17).

Il connaît uniquement : `Peer`, `Session`, `Message`, `Stream`.

---

# 2. Fondation : libp2p

Construit sur libp2p (implémentation Rust), qui fournit directement :

- **Découverte locale** : mDNS pour les pairs sur le même réseau.
- **Découverte distante** : DHT Kademlia pour retrouver un pair par son identifiant public sur Internet.
- **Chiffrement de transport** : protocole Noise (indépendant du chiffrement du contenu — défense en profondeur).
- **Traversée NAT** : relais + hole punching.

Ce choix évite de réimplémenter des briques non triviales (c'est l'essentiel de l'effort d'ingénierie de projets comme Tox ou Briar).

---

# 3. Relais

**Décidé : relais publics de libp2p pour la V1.** L'option d'un relais auto-hébergeable par l'utilisateur reste ouverte comme amélioration ultérieure — le protocole de relais de libp2p ne distingue pas relais public et relais auto-hébergé, donc cette évolution n'implique aucune refonte.

---

# 4. Sessions réseau

Machine d'état (cf. `003-architecture.md` Partie 3, §26) :

```
Disconnected → Connecting → Authenticated → Negotiating
→ Ready → Streaming → Closing → Disconnected
```

Une erreur réseau sur une session n'affecte jamais les autres sessions actives.

---

# 5. Rôle de pair "hôte"

Rôle optionnel, configurable **par coffre** (un appareil peut être hôte pour un coffre et simple pair pour un autre) :

- **Disponibilité garantie** : reste connecté en continu, servant de point de rendez-vous stable quand les autres pairs ne sont pas en ligne simultanément.
- **Réplique complète et prioritaire** : maintient toujours une copie intégrale à jour, contrairement à un pair mobile qui peut ne synchroniser qu'une partie du coffre.
- Ne remplace pas le P2P direct : deux pairs classiques en ligne simultanément continuent de synchroniser directement entre eux.
- Implémentation : un flag de rôle en configuration locale + une priorité de connexion (les autres pairs tentent l'hôte en premier, en complément de la découverte normale).

---

# 6. Sessions de manifest et invalidation

Le manifest n'est jamais conservé déchiffré à long terme côté distant : téléchargé chiffré, déchiffré localement pour l'usage, puis supprimé une fois l'usage terminé. Ce mécanisme gère la distribution et la mise à jour du manifest sans jamais le garder en clair sur le réseau.

## 6.1 Ouverture de session

Un pair B qui veut consulter un coffre envoie une requête au pair source (propriétaire connecté, ou pair hôte s'il existe). Le pair source répond avec le manifest enveloppé (cf. `013-security.md` pour le format de l'enveloppe) et un jeton de session.

## 6.2 Session active = abonnement

Tant que la session est active, le pair source considère B comme abonné aux mises à jour de ce coffre. Un heartbeat périodique sert de **filet de sécurité** contre les déconnexions brutales (crash, perte réseau) ; l'absence de heartbeat au-delà d'un timeout ferme la session automatiquement.

## 6.3 Notification de changement

Si une modification survient côté source pendant qu'une session est ouverte, le pair source pousse immédiatement une nouvelle racine Merkle (ou un manifest ré-enveloppé) vers les pairs abonnés — pas de réinterrogation périodique nécessaire côté B.

## 6.4 Fermeture de session

**Déclencheur principal : un événement de cycle de vie applicatif**, pas seulement le timeout. La perte de focus de l'app / sa mise en arrière-plan / sa fermeture côté B déclenche immédiatement l'envoi du message de fermeture de session et la suppression de la copie déchiffrée locale. Le timeout du §6.2 ne sert que de rattrapage en cas de crash ou de perte de connexion avant l'envoi du message.

---

# 7. Identité réseau : structure compte/appareil

Cf. `011-identity.md` pour le détail complet. Rappel des points qui touchent directement le Network Engine :

- Le **PeerId réseau réel** correspond à la clé d'appareil (`DeviceKey`), jamais à la clé de compte (`AccountKey`), qui ne sert qu'à signer les certificats d'appareil.
- Un contact partagé = la clé de compte (en-tête) ; les appareils valides pour ce compte sont découverts dynamiquement (DHT), chacun vérifiable par sa signature — pas besoin de ré-ajouter un contact à chaque nouvel appareil.

---

# 8. Mono-démon partagé par l'écosystème

**Décision : un seul démon réseau par appareil**, partagé par toutes les applications de l'écosystème (partage de fichiers, messagerie, futures apps) via plusieurs protocoles applicatifs multiplexés sur un même `Swarm` libp2p (un seul PeerId, un identifiant de protocole distinct par application — ex. `/syfi/files/1.0.0`, `/syfi/messenger/1.0.0`).

Raisons :

- Efficacité réseau (une seule présence DHT, un seul jeu de connexions/NAT traversal à maintenir) — significatif sur mobile.
- La résilience recherchée par un modèle multi-démon s'obtient à moindre coût par une supervision robuste du démon unique (redémarrage automatique, isolation interne des moteurs en modules Rust séparés) plutôt que par duplication complète.
- L'isolation de sécurité entre applications clientes reste garantie par la couche de capacités de l'IPC (`012-ipc.md`), pas par la séparation des processus réseau.

Ce choix n'est pas figé : la structure compte/appareil (`011-identity.md`) permet, si nécessaire, de faire tourner plusieurs démons (plusieurs sous-clés d'appareil certifiées sous le même compte) sans changement de schéma d'identité — c'est un choix de déploiement, pas une bifurcation d'architecture.

---

# 9. Événements

```
PeerDiscovered
PeerConnected
PeerDisconnected
ManifestSessionOpened
ManifestSessionClosed
ManifestPushed
RelayUsed
```

---

# 10. Erreurs

```
ConnectionFailed
NatTraversalFailed
SessionTimeout
UnauthorizedPeer
ProtocolMismatch
```

Toutes héritent de `NetworkError`.

---

# 11. Invariants

- Le Network ne connaît jamais le contenu des blocs ni du manifest en clair.
- Une session réseau compromise est immédiatement isolée, sans affecter les autres.
- Le rôle de pair hôte est spécifique à un coffre, jamais global à l'appareil.
- Une session de manifest fermée ne peut plus recevoir de notification de mise à jour tant qu'elle n'est pas rouverte.

---

# Conclusion

Le Network Engine reste une couche de transport pure, capable d'évoluer (changement de bibliothèque réseau, ajout d'un relais auto-hébergé, passage à un modèle multi-démon) sans jamais exposer sa complexité aux moteurs supérieurs, conformément au principe d'architecture hexagonale de `003-architecture.md` Partie 4.
