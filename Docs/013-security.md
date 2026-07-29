# docs/013-security.md

# Sécurité

Version : 2.0 (Draft)

---

# 1. Objectif

Ce document consolide l'ensemble des décisions de sécurité de SyFi, réparties ailleurs par moteur. Il sert de référence unique pour le modèle de menace, les primitives cryptographiques approuvées et les limites connues du système.

---

# 2. Modèle de menace

Cf. `000-introduction.md` et `001-vision.md` : SyFi adopte une posture Zero Trust.

- Chaque pair est potentiellement hostile.
- Chaque transfert est considéré comme observable.
- Chaque stockage distant est considéré comme compromis.
- La sécurité repose uniquement sur les primitives cryptographiques, jamais sur l'infrastructure.

---

# 3. Primitives approuvées (NFR-004)

Aucun algorithme "maison" n'est autorisé. Liste des primitives retenues :

| Usage | Primitive |
|---|---|
| Dérivation de clé (passphrase) | Argon2id |
| Dérivation de sous-clés (contexte) | HKDF-SHA256 |
| Chiffrement symétrique authentifié | XChaCha20-Poly1305 |
| Signature | Ed25519 |
| Échange/scellement asymétrique | X25519 (construction *sealed box*) |
| Intégrité (checksum, non cryptographique-critique) | BLAKE3 |

XChaCha20-Poly1305 est préféré à AES-GCM : nonce 192 bits (marge de sécurité bien plus large qu'AES-GCM en 96 bits pour une génération aléatoire), et performance logicielle homogène desktop/mobile sans dépendance à une accélération matérielle AES.

---

# 4. Gestion des clés du coffre

- Clé maître dérivée d'une passphrase utilisateur via Argon2id, elle-même chiffrant/déchiffrant les sous-clés du coffre.
- Sous-clés dérivées par HKDF avec un **contexte distinct** pour le manifest et pour les blocs de contenu (`"manifest"` vs `"blocks"`) — un seul secret à gérer, deux usages découplés.
- Rotation des clés : prévue comme exigence SHOULD (NFR-005), pas bloquante pour la V1, mais l'architecture ne doit jamais figer l'hypothèse "une seule clé pour toujours".

---

# 5. Enveloppe hybride de distribution du manifest

Le manifest est téléchargé chiffré, déchiffré uniquement en local sur l'appareil distant, jamais en clair sur le réseau, et supprimé localement après usage (cf. `010-network.md` §6).

- Le **contenu du manifest** est chiffré avec une **clé de session symétrique aléatoire** en XChaCha20-Poly1305.
- Cette clé de session (32 octets) est **chiffrée individuellement pour chaque pair autorisé** via une construction *sealed box* (X25519 + XSalsa20-Poly1305) — le coût de l'opération asymétrique ne porte que sur ces 32 octets, jamais sur le contenu du manifest lui-même.
- **Pourquoi pas de l'asymétrique pur** : les algorithmes asymétriques (RSA, ECC) sont largement plus lents par octet que le symétrique, et RSA impose une limite de taille de message. L'enveloppe hybride donne la propriété "chiffrement à clé asymétrique par destinataire" sans payer ce coût sur un fichier potentiellement volumineux.
- **Révocation** : à chaque réémission de manifest, le pair source ré-enveloppe simplement la clé de session pour chaque pair actuellement abonné — révoquer l'accès futur d'un pair revient à ne plus lui ré-envelopper la clé, sans avoir à rechiffrer quoi que ce soit rétroactivement.

---

# 6. Confidentialité des métadonnées (NFR-003)

Les noms de fichiers et la hiérarchie des dossiers ne sont **jamais visibles en clair sur le disque** ni sur le réseau. Le stockage physique ne révèle que des blocs opaques identifiés par UUID (cf. `006-storage.md`) ; la vraie arborescence n'existe que dans le manifest chiffré.

---

# 7. Adressage des blocs et absence de déduplication

Cf. `006-storage.md` §2 pour la décision et sa justification complète : les blocs sont adressés par UUID opaque généré à l'écriture, jamais par une empreinte de leur contenu. Un adressage par contenu impliquerait soit un chiffrement convergent (nonce déterministe dérivé du contenu en clair), qui expose le stockage physique à des attaques de confirmation de fichier, soit un hash du contenu chiffré, qui ne permettrait de toute façon aucune déduplication réelle (nonce aléatoire → empreintes différentes pour un même contenu). Ce compromis n'a pas été demandé et reste explicitement écarté pour la V1.

---

# 8. Ouverture du format (NFR contrainte d'écosystème)

Le coffre doit rester déchiffrable indépendamment de l'application, dès lors qu'on possède la passphrase — comme cryfs, gocryptfs ou un volume LUKS. Cela implique :

- Le format du coffre (dérivation de clé, structure des blocs, structure du manifest — cf. `014-file-format.md`) est **entièrement documenté** dans une spécification publique versionnée.
- Les primitives retenues (§3) sont **largement supportées par des outils tiers courants** (libsodium notamment), permettant en théorie l'écriture d'un script de déchiffrement autonome indépendant du code de l'application.
- Livrable prévu en parallèle du développement du Storage/Vault Engine : un **outil de déchiffrement de référence minimal**, autonome, hors de l'application principale.

---

# 9. Hygiène mémoire

- Aucune clé secrète n'est journalisée.
- Aucun mot de passe n'est conservé en mémoire plus longtemps que nécessaire.
- Les buffers contenant des données sensibles sont explicitement nettoyés (zeroization) lorsqu'ils ne sont plus utilisés (cf. `003-architecture.md` Partie 5, §73).

---

# 10. Surface d'attaque et limites connues

Cette section documente honnêtement ce que la conception actuelle **ne** protège **pas**, pour éviter toute promesse non tenue :

- Un pair ou un relais observant le trafic réseau peut mesurer la **taille**, la **fréquence** et le **timing** des échanges, même sans en déchiffrer le contenu — ce qui peut, dans certains scénarios, révéler des informations comportementales (volume d'activité, horaires de connexion). Aucune contre-mesure de type *traffic padding* n'est prévue pour la V1.
- Un pair hôte (`010-network.md` §5) détient nécessairement la clé maître du coffre pour servir une réplique complète — sa compromission expose l'intégralité du coffre qu'il héberge.
- La révocation d'un appareil (§5 de `011-identity.md`) empêche les futures connexions, mais ne retire pas rétroactivement l'accès aux données déjà synchronisées localement sur cet appareil avant révocation.

---

# 11. Révocation (vue d'ensemble)

Trois mécanismes de révocation distincts, à ne pas confondre :

- **Appareil** : certificat non renouvelé ou liste de révocation signée par la clé de compte (`011-identity.md`).
- **Pair (accès à un coffre)** : arrêt du ré-enveloppement de la clé de session du manifest (§5).
- **Application cliente locale (capacité IPC)** : suppression de l'entrée dans la table de capacités du démon (`012-ipc.md` §4.3).

---

# 12. Invariants

- Toutes les données persistantes sont chiffrées (NFR-001), sans exception.
- Toutes les données sont authentifiées (NFR-002).
- Aucun algorithme cryptographique non reconnu n'est utilisé (NFR-004).
- Une clé de signature n'est jamais réutilisée pour du chiffrement.
- Le format du coffre reste documenté et ouvert à toute implémentation indépendante.

---

# Conclusion

La sécurité de SyFi repose exclusivement sur des primitives éprouvées et sur une architecture qui ne fait jamais reposer la confidentialité sur l'infrastructure réseau ou sur la confiance envers un pair. Les limites listées en §10 ne sont pas des failles mais des choix de compromis assumés, à réévaluer explicitement si le contexte d'usage l'exige.
