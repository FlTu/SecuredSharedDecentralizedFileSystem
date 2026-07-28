# docs/001-vision.md

# Vision du projet

Version : 2.0 (Draft)

---

# 1. Objectif

SyFi est une plateforme de stockage distribué dont l'objectif est de permettre à un utilisateur de conserver la pleine propriété de ses données sans dépendre d'un fournisseur tiers.

Le projet ne cherche pas à reproduire un service de stockage cloud traditionnel. Il vise à proposer une alternative où les données, les identités et les mécanismes de synchronisation restent entièrement sous le contrôle de leurs propriétaires.

Le réseau constitue un moyen de communication. Il ne constitue jamais une autorité de confiance.

---

# 2. Les problèmes actuels

Les solutions de stockage existantes présentent plusieurs limites.

## Centralisation

La majorité des services imposent un serveur maître.

Celui-ci :

- possède les comptes utilisateurs ;
- contrôle les autorisations ;
- conserve les métadonnées ;
- devient un point de défaillance unique.

Même lorsque les données sont chiffrées, la perte du serveur rend souvent le service inutilisable.

---

## Confidentialité limitée

De nombreuses solutions protègent le contenu des fichiers mais exposent :

- les noms des fichiers ;
- leur taille ;
- l'arborescence ;
- la fréquence des modifications ;
- les appareils utilisés ;
- les horaires de connexion.

Ces informations permettent déjà d'établir un profil très précis d'un utilisateur.

SyFi considère également ces métadonnées comme sensibles.

---

## Dépendance à un fournisseur

La plupart des plateformes reposent sur :

- un abonnement ;
- une infrastructure privée ;
- un protocole propriétaire ;
- un format de stockage fermé.

Changer de fournisseur implique souvent une migration complexe.

SyFi adopte le principe inverse :

Le format appartient aux utilisateurs.

Jamais à l'application.

---

## Absence d'interopérabilité

Peu de solutions documentent complètement leur format.

Une implémentation indépendante est souvent impossible.

SyFi impose que chaque format soit entièrement documenté.

---

# 3. Vision

SyFi doit devenir une plateforme de confiance zéro (Zero Trust).

Chaque composant est considéré comme potentiellement compromis.

Le système reste néanmoins capable de préserver :

- la confidentialité ;
- l'intégrité ;
- l'authenticité ;
- la disponibilité.

La confiance ne repose jamais sur :

- un serveur ;
- un fournisseur ;
- un administrateur.

Elle repose exclusivement sur :

- les clés cryptographiques ;
- les signatures ;
- les preuves d'intégrité.

---

# 4. Cas d'utilisation

## Synchronisation personnelle

Un utilisateur possède :

- un ordinateur portable ;
- un ordinateur fixe ;
- un téléphone.

Les trois appareils doivent rester synchronisés sans serveur central.

---

## Sauvegarde personnelle

Un coffre est répliqué :

- sur plusieurs appareils ;
- sur un NAS ;
- sur un disque externe.

La perte d'un appareil ne provoque aucune perte de données.

---

## Partage privé

Deux utilisateurs souhaitent partager un dossier.

Ils échangent uniquement leurs identités.

Aucun compte sur un serveur n'est nécessaire.

---

## Travail hors ligne

Deux appareils modifient le même coffre.

La synchronisation est effectuée ultérieurement.

Les conflits sont détectés automatiquement.

---

## Réseau local

Deux appareils présents sur le même réseau découvrent automatiquement leurs voisins.

La synchronisation utilise directement le LAN.

Aucun trafic extérieur n'est requis.

---

## Réseau Internet

Deux appareils situés derrière des NAT doivent pouvoir communiquer.

Lorsque cela est possible :

- hole punching ;
- connexion directe.

Sinon :

- relais chiffré.

---

# 5. Personae

## Utilisateur individuel

Souhaite remplacer Dropbox.

Priorités :

- simplicité ;
- confidentialité ;
- synchronisation.

---

## Professionnel

Synchronise plusieurs postes de travail.

Souhaite maîtriser totalement son infrastructure.

---

## Développeur

Intègre SyFi dans une autre application.

Utilise :

- l'IPC ;
- les API publiques ;
- le démon.

---

## Administrateur

Déploie SyFi sur plusieurs machines.

Automatise les sauvegardes.

Surveille la synchronisation.

---

# 6. Objectifs fonctionnels

La version 1 doit permettre :

✓ créer un coffre

✓ ouvrir un coffre

✓ verrouiller un coffre

✓ ajouter des fichiers

✓ supprimer des fichiers

✓ déplacer des fichiers

✓ synchroniser plusieurs appareils

✓ partager un coffre

✓ gérer plusieurs identités

✓ détecter automatiquement les modifications

✓ restaurer après une interruption

---

# 7. Objectifs non fonctionnels

## Sécurité

Toutes les données sont chiffrées.

Aucune exception.

---

## Performance

Le système doit pouvoir gérer :

- plusieurs centaines de milliers de fichiers ;
- plusieurs millions de blocs.

Sans dégradation majeure.

---

## Portabilité

Le Core est entièrement écrit en Rust.

Il doit fonctionner sur :

- Linux
- Windows
- Android

Sans modification fonctionnelle.

---

## Résilience

Aucune opération critique ne doit laisser le coffre dans un état incohérent.

Toutes les écritures importantes sont transactionnelles.

---

## Maintenabilité

Chaque module possède une responsabilité unique.

Le couplage est réduit au minimum.

Les interfaces publiques sont documentées.

---

# 8. Contraintes

Le projet refuse volontairement :

- toute dépendance à un cloud propriétaire ;
- toute base de données distante obligatoire ;
- tout protocole fermé ;
- tout format binaire non documenté.

---

# 9. Critères de réussite

La première version est considérée comme réussie si :

- un coffre peut être créé ;
- plusieurs appareils peuvent être synchronisés ;
- la synchronisation fonctionne sans serveur central ;
- un coffre peut être partagé ;
- les données restent lisibles pendant plusieurs années malgré les évolutions logicielles.

---

# 10. Évolutions prévues

L'architecture doit permettre d'ajouter ultérieurement :

- synchronisation sélective ;
- historique complet ;
- snapshots ;
- stockage distribué multi-pairs ;
- collaboration temps réel ;
- moteur de recherche chiffré ;
- messagerie sécurisée ;
- appels audio/vidéo ;
- plugins ;
- API publique complète.

Aucune de ces évolutions ne devra nécessiter une refonte de l'architecture.

---

# Conclusion

La mission de SyFi n'est pas de créer une nouvelle application de synchronisation.

La mission est de construire une plateforme de stockage distribuée, sécurisée et extensible, destinée à devenir la fondation commune d'un écosystème d'applications fonctionnant sans autorité centrale.

Toutes les décisions d'architecture décrites dans les documents suivants devront être cohérentes avec cette vision.