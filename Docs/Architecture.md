# Architecture — Application de partage de fichiers chiffré P2P

*Nom de code : SyFi Depach(Systeme de Fichier Decentralisé Partagé Chiffré)
	       

## 1. Objectifs et périmètre

Créer une application qui :

- présente un **répertoire virtuel** reconstruit à partir d'un **stockage physique chiffré** (façon cryfs/gocryptfs) ;
- **synchronise** ce répertoire entre plusieurs appareils appartenant à un même utilisateur, ou partagés avec des contacts, via un réseau **pair-à-pair** sans serveur central obligatoire ;
- s'identifie via une **paire de clés cryptographiques** (pas de compte, pas d'e-mail, pas de serveur d'authentification) ;
- se **compile depuis un unique poste Linux** vers trois cibles : Linux, Windows, Android.

Hors périmètre pour la V1 (à réévaluer plus tard) : partage à des groupes larges (>10 pairs actifs simultanés), résolution de conflits automatique intelligente au-delà du "garder les deux versions", interface web/iOS, déduplication inter-fichiers.

## 2. Vue d'ensemble en couches

```
┌─────────────────────────────────────────────────────────┐
│  UI (par plateforme)                                     │
│  - Linux/Windows : Tauri (webview) ou egui (natif)        │
│  - Android : Kotlin, appel du core via JNI                │
├─────────────────────────────────────────────────────────┤
│  Core Rust (une seule codebase, compilée sur les 3 cibles)│
│  ┌───────────────┐ ┌───────────────┐ ┌─────────────────┐ │
│  │ Vault Engine   │ │ Index Engine  │ │ Sync Engine     │ │
│  │ (chiffrement,  │ │ (arborescence,│ │ (comparaison    │ │
│  │  blocs, clés)  │ │  manifeste)   │ │  Merkle, diff)  │ │
│  └───────────────┘ └───────────────┘ └─────────────────┘ │
│  ┌───────────────────────────────────────────────────────┐│
│  │ Network Engine (libp2p) : identité, découverte,        ││
│  │ transport chiffré, transfert de blocs                  ││
│  └───────────────────────────────────────────────────────┘│
├─────────────────────────────────────────────────────────┤
│  Stockage physique local (fichiers opaques sur disque)     │
└─────────────────────────────────────────────────────────┘
```

Le Core Rust ne dépend d'aucune UI. C'est une bibliothèque (`cdylib` + `staticlib`) exposant une API C stable (FFI), consommée directement par Tauri/egui sur desktop et via `jni` sur Android. Toute la logique sensible (crypto, sync, réseau) vit à un seul endroit : aucune réimplémentation par plateforme.

## 3. Vault Engine — le volume chiffré

### 3.1 Découpage et chiffrement

- Chaque fichier logique est découpé en **blocs de taille fixe** (proposition : 4 Mio, ajustable).
- Chaque bloc est chiffré indépendamment en **XChaCha20-Poly1305** :
  - nonce 192 bits généré aléatoirement par bloc (espace assez grand pour exclure toute collision pratique, contrairement à AES-GCM en 96 bits) ;
  - pas de dépendance à une accélération matérielle AES, donc performance homogène desktop/mobile.
- Clé de chiffrement des blocs dérivée par **HKDF-SHA256** à partir d'une clé maître (elle-même dérivée d'un mot de passe utilisateur via **Argon2id**, paramètres à durcir selon le matériel cible).
- Chaque bloc chiffré est stocké physiquement sous un **nom opaque** (UUID v4 ou hash du contenu chiffré) — le nom réel du fichier et sa position dans l'arborescence n'existent **que** dans l'index chiffré (cf. §4). Ce point est un choix de conception fort : il protège les métadonnées (comme cryfs) au prix d'une indirection systématique.

### 3.2 Séparation manifest / blocs (décision confirmée)

Point tranché : le **manifest (index) est chiffré séparément du contenu des blocs**, avec sa propre enveloppe de chiffrement. Conséquences directes sur le protocole :

- Un appareil distant qui possède la clé du coffre n'a besoin de déchiffrer **que le manifest** pour lister l'arborescence, afficher les fichiers/dossiers disponibles, calculer les diffs Merkle, etc. — jamais besoin de toucher au contenu des blocs pour ces opérations.
- Le manifest contient la correspondance complète `chemin logique → liste de block_id` (déjà prévu en §4.2 via `BlockRef`), donc au moment d'un téléchargement, les **blocs chiffrés sont récupérés tels quels**, sans étape de déchiffrement intermédiaire — le déchiffrement du contenu n'a lieu que localement, à l'usage réel du fichier (ouverture, lecture via l'explorateur virtuel).
- En pratique le manifest reste petit et est déchiffré/re-chiffré fréquemment (à chaque modification de l'arborescence), alors que les blocs sont gros et déchiffrés rarement (seulement à l'accès) : les deux n'ont donc pas le même profil de performance, d'où l'intérêt de les traiter comme deux objets chiffrés distincts plutôt qu'un seul blob.
- Techniquement, manifest et blocs peuvent partager la même clé maître (dérivation HKDF avec un `context string` différent — `"manifest"` vs `"blocks"`), ce qui évite de gérer deux secrets séparés tout en gardant les deux opérations découplées.

### 3.2bis Enveloppe de transmission du manifest (chiffrement hybride)

Contrainte précisée : le manifest est **téléchargé chiffré, déchiffré uniquement en local** sur l'appareil distant, jamais en clair sur le réseau, et **supprimé localement une fois la session de consultation terminée** (cf. mécanisme de session, §5.5). Pour la distribution du manifest à un pair distant spécifiquement, priorité donnée à la sécurité et à la vitesse.

- **Chiffrement hybride (enveloppe), pas de l'asymétrique pur** : le contenu du manifest est chiffré avec une **clé de session symétrique aléatoire** en XChaCha20-Poly1305 (rapide, même primitive que pour les blocs) ; cette clé de session (32 octets) est ensuite **chiffrée individuellement pour chaque pair autorisé** via une construction asymétrique légère (*sealed box*, X25519 + XSalsa20-Poly1305 — la même famille de primitives que le reste, disponible nativement dans libsodium). Le coût de l'opération asymétrique ne porte que sur ces 32 octets, jamais sur le contenu du manifest lui-même : le manifest peut grossir sans impact sur le coût de l'enveloppe.
- **Clé asymétrique par identité** : chaque identité (§5.1) doit posséder, en plus de sa paire Ed25519 de signature, une **paire X25519 de chiffrement** dédiée — les deux usages (signer/chiffrer) sont volontairement séparés, c'est la pratique standard (ne jamais réutiliser une clé de signature pour du chiffrement). À gérer dans `identity-core` (§5.1bis).
- **Bénéfice direct pour la révocation et les mises à jour** : à chaque réémission de manifest (§5.5), le pair source ré-enveloppe simplement la nouvelle clé de session pour chaque pair actuellement abonné — révoquer l'accès futur d'un pair revient à ne plus lui ré-envelopper la clé, sans avoir à rechiffrer quoi que ce soit rétroactivement.
- **Distinction avec le chiffrement "at rest"** : cette enveloppe hybride concerne la **distribution à un pair donné** ; elle s'ajoute à la clé maître du coffre (§3.2, §3.5) qui reste symétrique et partagée entre tous les appareils/pairs autorisés du coffre, pour garantir la propriété "déchiffrable hors-ligne avec la seule passphrase" du §3.4 — les deux mécanismes coexistent, ils répondent à deux besoins différents (stockage durable vs. distribution contrôlée par pair).



Découpage en **blocs de taille fixe** (pas de chunking par contenu/rolling-hash pour l'instant) : c'est le choix le plus simple et le plus rapide à l'implémentation comme à l'exécution. Le chunking par contenu (meilleure déduplication sur modification partielle d'un gros fichier) est noté comme **amélioration future**, exposée éventuellement comme un curseur "performance ↔ finesse de chiffrement" dans les paramètres avancés — mais l'architecture de l'index (`BlockRef` en liste ordonnée) est déjà compatible avec les deux approches, donc ce changement futur n'imposera pas de refonte du format.

### 3.4 Format ouvert et interopérable (contrainte d'écosystème)

Contrainte forte, non négociable pour la suite : le coffre doit rester **déchiffrable indépendamment de l'application**, comme cryfs, gocryptfs ou un volume LUKS — dès lors qu'on a la passphrase, aucun outil propriétaire ni aucune dépendance à la survie du projet ne doit être nécessaire. Cela implique :

- Le format du coffre (dérivation de clé, structure des blocs, structure du manifest) doit être **entièrement documenté dans une spécification écrite**, versionnée, publiée à côté du code — pas seulement "implicite dans le code Rust".
- Choix de primitives **largement standard et déjà supportées par des outils tiers courants** : Argon2id (KDF) et XChaCha20-Poly1305 (chiffrement) sont tous deux implémentés dans libsodium, disponible dans quasiment tous les langages — un utilisateur pourrait en théorie écrire un script de déchiffrement autonome (Python + libsodium, ou OpenSSL pour certaines primitives) sans toucher au code de l'appli.
- Livrable à prévoir en parallèle du développement : un **outil de déchiffrement de référence minimal** (script autonome, hors de l'appli principale), qui sert à la fois de preuve que le format est réellement ouvert et de garde-fou contre toute dérive accidentelle vers un format propriétaire de fait.

### 3.5 Gestion des clés

- Une clé maître par "coffre" (vault), stockée chiffrée par un mot de passe local (Argon2id + XChaCha20-Poly1305).
- Le partage d'un coffre avec un contact = partage hors-bande de la clé maître (via un canal déjà authentifié : l'échange d'identité P2P du §5), pas de re-dérivation par mot de passe côté destinataire.
- Prévoir dès la V1 une possibilité de **rotation de clé** (même si non implémentée tout de suite) : ne pas coder en dur l'hypothèse "une seule clé pour toujours".

### 3.3 Montage / accès

- **Linux** : montage FUSE réel via le crate `fuser`. Le répertoire virtuel apparaît comme un vrai point de montage.
- **Windows** : montage virtuel via WinFsp (binding `winfsp-rs`), même principe.
- **Android** : **pas de FUSE réaliste sans root**. L'accès se fait via un explorateur de fichiers intégré à l'application, qui déchiffre à la demande (lecture/écriture de blocs individuels, pas de déchiffrement intégral en mémoire).

## 4. Index Engine — le manifeste

### 4.1 Rôle

L'index est l'unique source de vérité sur l'arborescence logique. Il est lui-même chiffré et versionné.

### 4.2 Structure (proposition, sérialisée en CBOR)

```
IndexEntry {
  path: String,              // chemin logique complet
  entry_type: File | Dir,
  size: u64,
  content_hash: [u8; 32],    // BLAKE3 du contenu en clair
  blocks: Vec<BlockRef>,     // liste ordonnée des blocs
  mtime: u64,
  version_vector: Map<PeerId, u64>,  // cf. §4.3
  deleted: bool,              // tombstone plutôt que suppression physique immédiate
}

BlockRef {
  block_id: Uuid,     // = nom physique opaque sur disque
  nonce: [u8; 24],
  block_hash: [u8; 32],
}
```

Choix : **CBOR plutôt que JSON** pour la compacité et la vitesse de parsing (l'index est lu/écrit très fréquemment), et parce qu'il supporte nativement les types binaires (hash, nonce) sans encodage base64 intermédiaire.

### 4.3 Versionnement et conflits

- Chaque entrée porte un **vecteur de version** (un compteur par PeerId ayant modifié le fichier), pas un simple timestamp — un timestamp seul ne permet pas de distinguer "modification concurrente" de "modification séquentielle", ce qui est le problème classique de tous les synchroniseurs naïfs.
- Politique de conflit V1 : si deux vecteurs de version sont **incomparables** (ni l'un ni l'autre ne domine l'autre), la résolution dépend du type de fichier — pour les fichiers texte, voir le **Merge Engine** (§4.4) ; pour tout le reste, on conserve les deux fichiers en renommant l'un des deux (`fichier (conflit sur <peer>, <date>).ext`), à charge pour l'utilisateur de trancher manuellement. C'est la politique la plus simple qui ne perd jamais de données silencieusement.
- **Suppressions = tombstones, pas d'effacement immédiat de l'entrée d'index.** Un fichier supprimé n'est pas retiré du manifest : le champ `deleted` passe à `true` sur une entrée qui porte elle-même un vecteur de version, exactement comme une modification normale. Sans ce mécanisme, un pair resté longtemps hors-ligne qui revient avec sa vieille copie du fichier n'aurait aucun moyen de savoir "ce fichier a été supprimé après ma dernière synchro" — il risquerait de faire réapparaître un fichier supprimé. Le tombstone "gagne" contre toute version antérieure à la suppression, et "perd" contre une modification réellement postérieure (comme n'importe quel conflit de version). **Durée de rétention fixée à 1 mois par défaut, personnalisable** (paramètre par coffre) — passé ce délai, l'entrée est purgée définitivement de l'index et ne peut plus déclencher de réapparition.

### 4.4 Merge Engine — résolution de conflit façon git pour les fichiers texte

Ajout demandé : pour les fichiers dont le contenu s'y prête (texte : `.txt`, code source, markdown, config, etc.), tenter une **fusion à trois voies (3-way merge)** façon git plutôt que de systématiquement dupliquer les deux versions en conflit.

- **Détection du caractère "texte"** : heuristique sur le contenu plutôt que sur l'extension seule (plus robuste et généralisable) — un fichier est traité comme fusionnable s'il est intégralement décodable en UTF-8 valide et ne contient pas d'octets nuls en quantité significative. Les fichiers binaires (images, archives, exécutables) retombent automatiquement sur la politique "garder les deux versions" du §4.3.
- **Ce qu'il faut pour un vrai 3-way merge** : en plus des deux versions en conflit, il faut la **version ancêtre commune** (le dernier état synchronisé avant que les deux pairs ne divergent). C'est le point de complexité principal : le format actuel (§4.2) ne garde que la dernière version de chaque fichier. Il faut donc, dès qu'une divergence est détectée entre deux vecteurs de version incomparables, **conserver temporairement les blocs de l'ancêtre commun** (identifiable via l'historique des vecteurs de version) jusqu'à résolution du conflit — pas un historique complet façon git, juste une rétention ciblée "ancêtre + les deux branches divergentes" tant que le conflit n'est pas résolu. Une fois résolu, seuls les blocs de la version fusionnée sont conservés.
- **Algorithme** : un diff3 ligne-à-ligne classique (équivalent à ce que fait `git merge` en interne) — fusion automatique des portions non-conflictuelles, et pour les portions où les deux branches modifient les mêmes lignes, insertion de marqueurs de conflit (`<<<<<<<` / `=======` / `>>>>>>>`) dans un fichier de travail proposé à l'utilisateur.
- **Deux issues possibles** :
  1. **Fusion propre** (pas de chevauchement de lignes modifiées) : le fichier fusionné est appliqué automatiquement, une nouvelle entrée d'index est créée avec un vecteur de version qui domine les deux branches — aucune intervention utilisateur nécessaire.
  2. **Fusion avec conflit résiduel** : le fichier avec marqueurs de conflit est présenté à l'utilisateur (dans l'UI, avec un affichage diff à deux colonnes ou inline) pour édition manuelle ; l'entrée d'index correspondante reste marquée "conflit non résolu" tant que l'utilisateur n'a pas validé une résolution.
- **Coût** : cette fonctionnalité n'est pas gratuite en complexité (gestion de l'ancêtre commun, UI de résolution dédiée) mais reste raisonnable si elle est **limitée aux fichiers texte détectés** — les gros fichiers binaires, qui sont la majorité du volume de données en pratique, ne sont jamais concernés et gardent la politique simple du §4.3. À prévoir comme itération après le Sync Engine de base (§8), pas dans le MVP initial.

## 5. Network Engine — identité et transport

### 5.1 Identité

- Chaque appareil génère une paire de clés **Ed25519** au premier lancement.
- La **clé publique encodée** (base58 ou base32, façon adresse Tox/Session) est l'identifiant que l'utilisateur partage à ses contacts.
- Ajout de contact = échange mutuel hors-bande de cette clé publique (QR code entre deux appareils, ou lien copié/collé).

### 5.1bis Identité comme brique d'écosystème (contrainte de convergence avec l'app de messagerie)

Changement de conception important : l'identité (§5.1) **ne doit pas être spécifique à cette application**. Puisqu'une app de messagerie est en développement en parallèle et doit pouvoir partager le même système d'identifiants, l'identité doit être traitée comme une **brique indépendante et réutilisable**, pas comme un détail d'implémentation de l'app de partage de fichiers :

- Extraire la génération de clé, l'encodage d'identifiant, et la logique de connexion pair-à-pair libp2p dans une **crate Rust séparée** (`identity-core` ou équivalent), sans aucune dépendance vers `vault-engine`/`index-engine`/`sync-engine`. Les deux applications (partage de fichiers et messagerie) consomment cette même crate.
- Un même identifiant (une même paire de clés) peut alors servir **à la fois** de PeerId pour la messagerie et pour le partage de fichiers — un contact ajouté une fois (dans l'une ou l'autre app) est valide pour les deux, exactement comme un identifiant Session sert pour plusieurs types d'échanges.
- Techniquement, cela s'obtient nativement avec libp2p : un même `Swarm` (identifié par un seul PeerId) peut faire tourner **plusieurs protocoles applicatifs en parallèle**, chacun avec son propre identifiant de protocole (`/monapp/fichiers/1.0.0`, `/monapp/messagerie/1.0.0`). Les deux apps peuvent soit partager un seul démon réseau, soit tourner en deux processus qui s'accordent sur la même paire de clés stockée localement — à trancher selon si tu veux un démon réseau unique partagé par tout l'écosystème ou un par application (cf. §7bis).
- Garder l'encodage d'identifiant volontairement simple et versionné (ex : préfixe de version + clé publique + checksum), pour pouvoir faire évoluer le format sans casser la compatibilité entre apps développées à des rythmes différents.

### 5.1ter Structure d'identifiant : couple compte/appareil (en réponse à ton point 4)

Problème soulevé, réel et important : l'app de messagerie permet de lier plusieurs appareils à un même compte (façon WhatsApp), mais aujourd'hui chaque appareil a un identifiant différent sous ce même compte — ce qui casse en partie l'idée d'un identifiant unique à partager, et rend absurde le partage de coffre entre plusieurs appareils d'un même utilisateur si chacun doit être ajouté comme un contact séparé.

**Décision proposée : la 2ᵉ option que tu envisages, un identifiant construit en deux parties (en-tête compte + corps appareil), pas deux identifiants juxtaposés.**

- **Clé de compte** : une paire Ed25519 de long terme, générée une seule fois, qui représente l'utilisateur — elle ne sert **jamais directement** de PeerId réseau (elle ne se connecte à rien en tant que telle), son seul rôle est de **signer** les clés d'appareil qui en dépendent. C'est le même principe que la clé d'identité de Signal qui signe les pré-clés d'appareil, ou une petite autorité de certification personnelle.
- **Clé d'appareil** : chaque appareil génère sa propre paire Ed25519 au premier lancement (comme décrit en §5.1) — c'est **elle** qui sert de PeerId réseau réel pour ce périphérique. Elle est accompagnée d'un **certificat** (la clé publique d'appareil + une signature par la clé de compte) prouvant son appartenance au compte.
- **Identifiant partagé avec un contact = la clé de compte (l'en-tête)**, pas une clé d'appareil individuelle. Quand un contact veut se connecter, il découvre dynamiquement (via la DHT, ou via l'échange initial) l'ensemble des appareils actuellement valides pour ce compte, chacun vérifiable par sa signature — pas besoin de ré-ajouter un contact à chaque nouvel appareil.
- Ce même mécanisme résout directement le partage de coffre entre plusieurs de tes propres appareils : ils partagent tous la même clé de compte en en-tête, donc "faire confiance à tous mes appareils" devient une seule règle (vérifier la signature de compte), plutôt qu'une liste à maintenir manuellement.
- Révocation d'un appareil compromis ou perdu = invalider son certificat (liste de révocation signée par la clé de compte, ou certificats à durée de vie limitée avec renouvellement périodique) — sans jamais toucher à la clé de compte elle-même.
- Cette structure vit dans `identity-core` (§5.1bis) : elle est donc directement réutilisable telle quelle par l'app de messagerie, ce qui répond au problème que tu as identifié à sa source plutôt que de le corriger a posteriori dans chaque app séparément.

### 5.1quater Mono-démon vs multi-démon : arbitrage

Ta demande d'aide à trancher — voici comment je vois le compromis, avec une recommandation.

- **Mono-démon** (un seul processus réseau par appareil, partagé par toutes les apps de l'écosystème via plusieurs protocoles applicatifs sur un même `Swarm`, §5.1bis) : le plus efficace — une seule présence DHT, une seule gestion de connexions/NAT traversal à maintenir, ce qui compte particulièrement sur mobile où chaque connexion active et chaque participation DHT coûte de la batterie.
- **Multi-démon** (un processus réseau distinct par app, chacun avec sa propre sous-clé d'appareil certifiée par la même clé de compte, §5.1ter) : meilleure isolation (un crash ou une compromission du démon de messagerie n'affecte pas le démon du coffre de fichiers), au prix d'une duplication complète des coûts réseau (deux présences DHT, deux jeux de connexions, deux fois le travail de NAT traversal).
- **Recommandation : mono-démon par défaut**, pour deux raisons concrètes : (1) le coût réseau dupliqué du multi-démon est significatif sur mobile, exactement le genre de compromis qui se remarque en usage réel (batterie, données) ; (2) la résilience recherchée par le multi-démon (redondance) s'obtient à moindre coût par une **supervision robuste d'un démon unique** (redémarrage automatique en cas de crash, isolation interne des moteurs Vault/Index/Sync et Messagerie en modules Rust séparés à l'intérieur du même processus) plutôt qu'en dupliquant tout le processus.
- **L'isolation de sécurité entre apps reste garantie autrement** : même dans un démon unique, la couche `ecosystem-ipc` (§6) contrôle par capacité ce que chaque app cliente peut faire — une compromission de l'app de messagerie côté client ne donne pas accès au coffre de fichiers, parce que l'accès est vérifié par jeton de capacité, pas par confiance implicite du fait de partager un processus.
- Grâce à la structure compte/appareil (§5.1ter), **ce choix n'est pas figé définitivement** : si l'expérience en usage réel montre que l'isolation prime sur l'efficacité pour un cas particulier, il est possible de faire tourner deux démons (deux sous-clés d'appareil différentes sous le même compte) sans changement de schéma d'identité — c'est un choix de déploiement, pas une bifurcation d'architecture.



Besoin exprimé : pouvoir configurer un appareil comme point central pour un ou plusieurs répertoires donnés — un rôle optionnel de **pair hôte**, distinct du pair "client" classique. Conception proposée :

- N'importe quel pair peut être promu "hôte" **pour un coffre donné** (la configuration est par coffre : un même appareil peut être hôte pour l'un et simple pair pour un autre).
- Un pair hôte a deux propriétés supplémentaires par rapport à un pair normal :
  1. **Disponibilité garantie** : il reste connecté en continu (démon, éventuellement sur un NAS ou un petit serveur perso), donc les autres pairs peuvent toujours le joindre pour synchroniser, même si aucun autre pair n'est en ligne au même moment — il devient le point de passage systématique plutôt que de dépendre d'une connexion directe éphémère entre deux appareils qui ne sont pas forcément en ligne en même temps.
  2. **Réplique complète et prioritaire** : contrairement à un pair mobile qui peut choisir de ne synchroniser qu'une partie du coffre (sync sélective, à prévoir), le pair hôte maintient toujours une copie intégrale à jour.
- Ce rôle ne remplace pas le P2P direct : quand deux pairs classiques sont en ligne en même temps, ils continuent de synchroniser directement entre eux (§5.4). L'hôte sert de filet de sécurité et de point de rendez-vous stable, pas de serveur central obligatoire — cohérent avec l'objectif "sans serveur central obligatoire" du §1.
- Implémentation : pas de nouveau protocole réseau nécessaire, juste un **flag de rôle** dans la configuration locale du coffre + une priorité de connexion (les autres pairs tentent de se connecter à l'hôte en premier s'il est déclaré, en complément de la découverte normale).

### 5.3 Transport et découverte

Construit sur **libp2p** (implémentation Rust mature), qui fournit directement :

- **Découverte locale** : mDNS pour les pairs sur le même réseau.
- **Découverte distante** : DHT Kademlia pour retrouver un pair par son identifiant public sur Internet.
- **Chiffrement de transport** : protocole Noise (indépendant du chiffrement du contenu du §3, défense en profondeur).
- **Traversée NAT** : relais + hole punching, pour les cas où les deux pairs sont derrière des box grand public. **Décidé : relais publics de libp2p pour la V1** ; l'option d'un relais auto-hébergeable par l'utilisateur reste ouverte comme amélioration ultérieure (aucune des deux options n'est fermée par ce choix — le protocole de relais de libp2p ne fait pas de distinction entre relais public et relais auto-hébergé).

Ce choix évite de réimplémenter ces briques (c'est tout l'historique de projets comme Tox ou Briar, non trivial à faire soi-même correctement).

### 5.4 Protocole de synchronisation (façon torrent)

1. Deux pairs connectés échangent la **racine d'un arbre de Merkle** construit sur les hashs des `IndexEntry`.
2. Si les racines diffèrent, ils descendent l'arbre pour isoler précisément **quelles entrées** diffèrent (pas besoin de comparer fichier par fichier un par un).
3. Pour chaque entrée divergente, comparaison des vecteurs de version → détermine qui a la version la plus récente, ou déclenche un conflit (§4.3).
4. Seuls les **blocs manquants** sont transférés (identifiés par leur `block_id`/hash) — et si le même dossier est partagé par plusieurs pairs, les blocs peuvent être récupérés en parallèle depuis plusieurs sources, comme un swarm torrent.

### 5.5 Sessions de manifest et invalidation (en réponse à ton point 1a)

Contrainte précisée : le manifest n'est jamais conservé déchiffré à long terme côté distant — téléchargé chiffré, déchiffré en local pour l'usage (navigation, calcul de diff), puis supprimé une fois l'usage terminé. Il faut donc un mécanisme pour que le pair source sache **quand** un distant a fini d'utiliser sa copie, afin de savoir à qui pousser une mise à jour si une modification survient entre-temps. Conception proposée, en extension légère du protocole Merkle du §5.4 (pas une brique from scratch) :

1. **Ouverture de session** : quand un pair B veut consulter un coffre, il envoie une requête au pair source (le propriétaire connecté, ou le pair hôte s'il existe, §5.2). Le pair source répond avec le manifest enveloppé (§3.2bis) et un **jeton de session** (identifiant aléatoire + une expiration par défaut, pour couvrir aussi les déconnexions brutales).
2. **Session active = abonnement** : tant que la session est active, le pair source considère B comme abonné aux mises à jour de ce coffre. Un heartbeat périodique léger (déjà porté par la connexion libp2p sous-jacente) sert de **filet de sécurité** contre les déconnexions brutales (crash, perte réseau) ; l'absence de heartbeat au-delà d'un timeout ferme la session automatiquement côté source.
3. **Notification de changement** : si une modification a lieu côté source pendant qu'une session est ouverte, le pair source **pousse immédiatement** vers tous les pairs abonnés une nouvelle racine Merkle (ou directement un manifest ré-enveloppé si le changement est significatif) — B n'a pas besoin de réinterroger périodiquement, il est notifié activement.
4. **Fermeture de session — déclencheur principal : un événement de cycle de vie applicatif, pas seulement le timeout.** Le signal qui compte le plus en pratique n'est pas "l'utilisateur a fini de lire" (difficile à détecter proprement) mais la **perte de focus de l'app / sa mise en arrière-plan / sa fermeture** côté B — ce sont des événements que chaque plateforme expose nativement (cycle de vie d'activité Android, événements de focus de fenêtre sur Linux/Windows). Dès que B détecte l'un de ces événements, il envoie immédiatement le message de fermeture de session et supprime sa copie déchiffrée en local — pas besoin d'attendre une action explicite type "bouton fermer". Le timeout du point 2 ne sert alors que de rattrapage pour les cas où l'événement de cycle de vie n'a pas pu être émis proprement (crash, perte de connexion avant l'envoi du message).

## 6. Intégration écosystème et API pour apps tierces

Contrainte exprimée : cette app n'est pas isolée, elle doit s'intégrer avec d'autres apps de l'écosystème (au moins l'app de messagerie), à la fois au niveau **identité** (traité en §5.1bis) et au niveau **fonctionnel** — d'autres apps doivent pouvoir accéder à certaines fonctions, voire à des fragments d'interface.

Proposition d'architecture pour ce besoin :

- Le Core Rust expose, en plus de son FFI direct (consommé par l'UI de *cette* app), une **API locale inter-processus** (IPC) : un socket Unix local sur Linux/Android, un named pipe sur Windows.
- **Choix tranché : protocole custom**, pas gRPC ni JSON-RPC générique — cohérent avec l'objectif de robustesse (efficacité + sécurité) et avec la volonté de le réutiliser/adapter dans les autres apps de l'écosystème :
  - **Sérialisation** : CBOR, pour rester cohérent avec le choix déjà fait pour le manifest (§4.2) — une seule bibliothèque de sérialisation dans tout le Core, pas de dépendance supplémentaire (protobuf/grpc apportent leur propre toolchain de génération de code, ce qu'on évite ici).
  - **Framing** : messages préfixés par leur longueur (u32) sur le socket local, pas de négociation HTTP/2 ni de couche de transport superflue — l'IPC est local, donc pas besoin du poids d'un vrai protocole réseau.
  - **Sécurité/robustesse** : deux niveaux. (1) Contrôle d'accès au socket lui-même via les permissions du système de fichiers (le socket Unix n'est lisible/inscriptible que par l'utilisateur courant) ou l'ACL du named pipe sous Windows. (2) Par-dessus, un système de **capacités explicites par app cliente**, détaillé ci-dessous.
  - **Schéma versionné** : chaque message porte un numéro de version de schéma, pour permettre à des apps de l'écosystème développées à des rythmes différents de rester compatibles sans forcer une mise à jour synchronisée.
  - Cette couche est conçue dès le départ comme une **crate séparée** (`ecosystem-ipc` ou équivalent, à côté de `identity-core`), justement pour être reprise telle quelle par l'app de messagerie plutôt que réimplémentée.

**Format du jeton de capacité (proposition concrète, puisque c'est un point technique mineur qui ne nécessite pas d'arbitrage de ta part)** :

- Un **jeton opaque aléatoire** (32 octets, pas de structure signée à vérifier — l'IPC est purement local, donc pas besoin de la complexité d'un JWT ou d'une structure signée cryptographiquement), généré au moment où une app cliente se fait connaître au Core pour la première fois (premier appairage).
- Le Core garde une **table de capacités locale** : `jeton → { nom de l'app, portées accordées, date de création }`. Les "portées" (*scopes*) sont un ensemble fixe et restreint défini à la compilation (ex : `contacts:read`, `coffres:list`, `coffre:contenu:<id>`, `sync:statut`) — une simple recherche dans une table, pas un langage de permissions à concevoir.
- Le jeton est stocké côté client dans le **stockage sécurisé natif de la plateforme** (Android Keystore, Windows Credential Manager, Secret Service/libsecret sous Linux), jamais en fichier plat.
- **Révocation** : une interface "applications connectées" côté utilisateur permet de retirer une entrée de la table à tout moment — l'app concernée perd l'accès immédiatement à sa prochaine requête, et doit se réappairer.
- Pas d'expiration automatique par défaut en V1 (le jeton reste valide jusqu'à révocation explicite) ; une expiration par inactivité pourra être ajoutée plus tard sans changer la structure.
- Fonctions à exposer en priorité via cette API : liste des contacts/identités connues, liste des coffres partagés et leur arborescence (déchiffrée via le manifest, cf. §3.2), statut de synchronisation. Le contenu des fichiers eux-mêmes reste un accès explicite (capacité dédiée), pas exposé par défaut à toute app qui interroge l'API.
- Pour le "donner accès à certaines parties de l'interface à ces autres apps" : plutôt que de dupliquer des écrans dans chaque app, prévoir des **composants d'UI réutilisables** (ex : un composant "sélecteur de contact" ou "explorateur de coffre partagé" packagé séparément — en Kotlin pour Android, en web-component ou crate Tauri pour desktop) que l'app de messagerie peut embarquer directement plutôt que de réimplémenter sa propre vue.
- Ce sujet reste le moins mature du document : la V1 doit se concentrer sur le Core et son API interne ; l'API IPC et les composants d'UI partagés peuvent être conçus dès le squelette de projet (les prévoir dans la structure de crates) mais implémentés seulement une fois l'app de messagerie elle-même prête à les consommer.

## 7. Cross-compilation depuis Linux

| Cible | Toolchain | Méthode |
|---|---|---|
| Linux x86_64 | native | `cargo build --release` |
| Windows x86_64 | `x86_64-pc-windows-gnu` | `cross` (via Docker) ou mingw-w64 installé localement |
| Android arm64 / armv7 | NDK | `cargo-ndk` |

Le Core Rust se compile en `cdylib` (`.so` pour Linux/Android, `.dll` pour Windows) consommée par la couche UI native de chaque plateforme. Un seul `Cargo.toml` workspace, avec les crates `vault-engine`, `index-engine`, `merge-engine`, `sync-engine`, `network-engine`, `identity-core`, `ecosystem-ipc`, `ffi` séparées pour permettre de tester chaque brique indépendamment.

## 8. Plan de développement proposé (par phases)

1. **Vault Engine seul** : chiffrement/déchiffrement de blocs, tests unitaires, CLI minimale pour valider le format sur disque.
2. **Index Engine** : lecture/écriture du manifeste, gestion des vecteurs de version, toujours en local (pas de réseau).
3. **Squelette cross-compilé** : vérifier que le Core compile et s'exécute sur les 3 cibles avant d'ajouter la complexité réseau ; y poser la structure de crates (dont `identity-core` séparée, §5.1bis).
4. **Network Engine** : identité, découverte LAN (mDNS) en premier, DHT ensuite.
5. **Sync Engine** : protocole Merkle + transfert de blocs, d'abord entre deux pairs sur LAN, avec la politique de conflit simple du §4.3 (garder les deux versions).
6. **Rôle pair hôte** (§5.2) : une fois le P2P direct stable entre deux pairs classiques.
7. **Merge Engine** (§4.4) : 3-way merge texte + UI de résolution — itération après un Sync Engine stable, pas dans le MVP.
8. **UI** par plateforme.
9. **`ecosystem-ipc` et composants d'UI partagés** (§6) : une fois l'app de messagerie prête à les consommer.

## 9. Points ouverts restants

- Rédaction de la spécification ouverte du format de coffre (§3.4) et de l'outil de déchiffrement de référence — à faire en parallèle du Vault Engine, pas après coup (accepté, reste un chantier à part entière).
- Valeur exacte du timeout de secours de session de manifest (§5.5) — le déclencheur principal (perte de focus/mise en arrière-plan) est tranché, seule la durée du filet de sécurité en cas de crash reste à calibrer empiriquement.
- Durée de validité et mécanisme de renouvellement du certificat d'appareil signé par la clé de compte (§5.1ter) : approche générale confirmée, durée exacte à spécifier avant implémentation.
- Liste exacte des portées (*scopes*) de capacité `ecosystem-ipc` (§6) : reporté, non urgent — à affiner une fois les besoins réels de l'app de messagerie mieux connus.
