# docs/011-identity.md

# Identity Engine

Version : 2.0 (Draft)

---

# 1. Objectif

L'Identity Engine gère les comptes, les appareils, les certificats et les signatures. Il ne connaît aucun coffre (cf. `003-architecture.md` §5).

Il constitue une brique **réutilisable par toute application de l'écosystème** (partage de fichiers, messagerie, futures applications) — pas une brique spécifique à SyFi.

---

# 2. Structure compte/appareil

Problème à résoudre : une identité "à plat" (une seule paire de clés par appareil) rend absurde le partage entre plusieurs appareils d'un même utilisateur, et casse la convergence entre applications si chaque appareil apparaît sous un identifiant différent pour un même compte.

**Décision : identifiant construit en deux parties (en-tête compte + corps appareil), pas deux identifiants juxtaposés.**

## 2.1 Clé de compte (`AccountKey`)

- Paire **Ed25519** de long terme, générée une seule fois.
- Ne sert **jamais directement de PeerId réseau** — son seul rôle est de **signer** les clés d'appareil qui en dépendent (même principe que la clé d'identité de Signal signant les pré-clés d'appareil, ou une petite autorité de certification personnelle).

## 2.2 Clé d'appareil (`DeviceKey`)

- Paire **Ed25519** générée par chaque appareil au premier lancement.
- Sert de **PeerId réseau réel** pour cet appareil.
- Accompagnée d'un **certificat** : clé publique d'appareil + signature par l'`AccountKey`, prouvant son appartenance au compte.

## 2.3 Identifiant partagé

L'identifiant qu'un utilisateur partage à un contact est la **clé de compte** (l'en-tête), jamais une clé d'appareil individuelle. Un contact découvre dynamiquement (via la DHT ou l'échange initial) l'ensemble des appareils actuellement valides pour ce compte, chacun vérifiable par sa signature.

Conséquence directe : le partage de coffre entre plusieurs appareils d'un même utilisateur devient une seule règle ("faire confiance à tous les appareils signés par ma clé de compte"), pas une liste à maintenir manuellement.

---

# 3. Paire de chiffrement compagnon

En plus de sa paire Ed25519 de signature, chaque identité (compte et appareil) possède une **paire X25519 de chiffrement** dédiée. Les deux usages (signer / chiffrer) sont volontairement séparés — ne jamais réutiliser une clé de signature pour du chiffrement.

Cette paire X25519 sert notamment à l'enveloppe hybride de distribution du manifest (cf. `013-security.md`).

---

# 4. Encodage de l'identifiant

- Encodage simple et **versionné** (préfixe de version du schéma + clé publique + checksum), en base58 ou base32 — façon adresse Tox/Session.
- Volontairement minimal, pour permettre à des applications de l'écosystème développées à des rythmes différents de rester compatibles sans mise à jour synchronisée forcée.
- Ajout de contact = échange mutuel hors-bande de cet identifiant (QR code entre deux appareils, ou lien copié/collé — cf. `004-workspace.md` §14, crate `identity`).

---

# 5. Certificat d'appareil

Contenu minimal :

```
DevicePublicKey
AccountPublicKey
IssuedAt
ExpiresAt
Signature (par AccountKey)
```

- Durée de validité limitée avec renouvellement périodique (valeur exacte à calibrer — cf. `015-roadmap.md`, point ouvert).
- Révocation : liste de révocation signée par la clé de compte, ou expiration naturelle du certificat sans renouvellement — sans jamais toucher à la clé de compte elle-même.

---

# 6. Réutilisation par l'écosystème

Cette crate (`identity`) ne dépend d'aucune autre crate du Core (`vault`, `manifest`, `sync`, `storage`) — uniquement de `crypto` et `common`. Toute application de l'écosystème (y compris une future application de messagerie) consomme la même crate, ce qui garantit qu'un identifiant de compte reste valide et reconnu par toutes les applications qui partagent le même démon (cf. `010-network.md` §8, mono-démon).

---

# 7. Multi-appareils, multi-identités

Une identité peut posséder plusieurs appareils (cf. `003-architecture.md` Partie 3 §36) ; chaque appareil possède un identifiant unique, une paire de clés dédiée, des métadonnées, et un état de confiance. La révocation d'un appareil ne remet pas en cause les autres.

Le démon peut gérer plusieurs identités simultanément (§35 du même document) ; chaque identité possède ses clés, ses appareils, ses autorisations, isolés les uns des autres.

---

# 8. Événements

```
IdentityCreated
DeviceAdded
DeviceRevoked
DeviceCertificateRenewed
ContactAdded
```

---

# 9. Erreurs

```
InvalidCertificate
ExpiredCertificate
UnknownAccount
RevokedDevice
SignatureVerificationFailed
```

Toutes héritent de `IdentityError`.

---

# 10. Invariants

- L'`AccountKey` ne sert jamais de PeerId réseau.
- Un `DeviceKey` sans certificat valide n'est jamais accepté par un pair.
- Une paire de signature n'est jamais réutilisée pour du chiffrement.
- La révocation d'un appareil n'invalide jamais les autres appareils du même compte.
- L'Identity Engine ne connaît jamais la structure d'un coffre.

---

# Conclusion

En séparant clé de compte et clé d'appareil, et en gardant la crate `identity` totalement indépendante du reste du Core, SyFi résout à la fois le partage de coffre entre appareils d'un même utilisateur et la convergence des identifiants avec les futures applications de l'écosystème — sans qu'aucune de ces applications n'ait à réimplémenter sa propre gestion d'identité.
