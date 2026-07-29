# docs/005-daemon.md

# Daemon Architecture

Version : 2.0 (Draft)

---

# 1. Objectif

Le Daemon constitue le point d'entrée unique de SyFi.

Toutes les applications (Desktop, CLI, Android et futures applications)
communiquent exclusivement avec lui.

Le Daemon :

- ne chiffre pas les données ;
- ne synchronise pas directement les fichiers ;
- ne gère pas le stockage.

Son rôle consiste uniquement à orchestrer les services du Core.

---

# 2. Philosophie

Le Daemon est construit comme un microkernel.

Chaque fonctionnalité est implémentée sous forme d'un service indépendant.

Le microkernel fournit :

- le cycle de vie des services ;
- la communication inter-services ;
- les sessions ;
- les permissions ;
- l'IPC ;
- le scheduler ;
- l'Event Bus.

---

# 3. Vue d'ensemble

```

+------------------------------------------------+

Applications

Desktop

Android

CLI

Future Messenger

Plugins

+-------------------+----------------------------+

IPC

+-------------------v----------------------------+

MICROKERNEL

|

+----------------------------------------------+

Vault Service

Identity Service

Network Service

Sync Service

Notification Service

Scheduler Service

Configuration Service

Capability Service

+----------------------------------------------+

|

Core

```

---

# 4. Les responsabilités

Le Daemon est responsable de :

- démarrer les services ;
- arrêter les services ;
- surveiller leur état ;
- distribuer les événements ;
- gérer les connexions IPC ;
- gérer les permissions ;
- surveiller les ressources.

Le Daemon n'exécute aucune logique métier.

---

# 5. Les Services

Chaque service possède :

- un identifiant ;
- un état ;
- un contexte ;
- un canal d'événements.

Tous les services utilisent le même contrat.

```

trait Service {

fn start()

fn stop()

fn pause()

fn resume()

fn health()

}

```

---

# 6. Cycle de vie

Tous les services suivent la même machine d'état.

```

Created

↓

Starting

↓

Running

↓

Paused

↓

Running

↓

Stopping

↓

Stopped

```

En cas d'erreur :

```

Running

↓

Failed

↓

Restarting

↓

Running

```

---

# 7. Service Vault

Responsabilités :

- ouvrir un coffre ;
- fermer un coffre ;
- verrouiller ;
- déverrouiller ;
- gérer les handles.

Il délègue toutes les opérations au Vault Engine.

---

# 8. Service Identity

Responsabilités :

- charger les identités ;
- gérer les appareils ;
- importer/exporter les certificats ;
- fournir les signatures.

---

# 9. Service Network

Responsabilités :

- écouter les connexions ;
- découvrir les pairs ;
- gérer les relais ;
- gérer les sessions.

Il ne manipule jamais les fichiers.

---

# 10. Service Sync

Responsabilités :

- lancer les synchronisations ;
- surveiller leur progression ;
- reprendre après interruption ;
- publier les événements.

---

# 11. Service Scheduler

Le Scheduler gère les tâches.

Toutes les opérations importantes deviennent des Jobs.

Exemple :

```

Import File

↓

Job

↓

Queue

↓

Worker

↓

Completed

```

Les Jobs possèdent :

- un UUID ;
- une priorité ;
- un propriétaire ;
- un état.

---

# 12. Service Configuration

Responsabilités :

- charger la configuration ;
- la valider ;
- la diffuser aux services.

Aucun service ne lit directement un fichier.

---

# 13. Service Notification

Diffuse les événements.

Exemple :

```

Sync Started

Sync Progress

Sync Completed

Peer Connected

Vault Locked

```

Les interfaces graphiques s'abonnent simplement à ces événements.

---

# 14. Capability Service

Le démon ne donne jamais un accès global.

Chaque client reçoit des capacités.

Exemple :

```

ReadVault

WriteVault

ShareVault

ManageIdentity

NetworkAdmin

```

Les capacités sont :

- limitées ;
- révocables ;
- auditables.

---

# 15. Sessions IPC

Chaque connexion possède une Session.

```

Desktop

↓

IPC

↓

Session

↓

Capabilities

↓

Services

```

Une session connaît :

- le client ;
- son identité ;
- ses capacités ;
- son état.

---

# 16. Gestion des capacités

Chaque requête suit le même pipeline.

```

Request

↓

Session

↓

Capability Check

↓

Authorization

↓

Service

↓

Response

```

Aucun service ne réalise lui-même les vérifications d'accès.

---

# 17. Gestion des coffres

Le Daemon conserve une table des coffres ouverts.

```

Vault Registry

├── Vault A

├── Vault B

├── Vault C

└── Vault D

```

Chaque entrée possède :

- son contexte ;
- son état ;
- son scheduler ;
- son cache.

---

# 18. Gestion des identités

Même principe.

```

Identity Registry

├── Alice

├── Bob

└── Company

```

Une identité peut être utilisée simultanément par plusieurs coffres.

---

# 19. Gestion des pairs

```

Peer Registry

├── Device A

├── Device B

├── Device C

└── Device D

```

Le registre contient uniquement les informations temporaires.

Les certificats restent dans Identity.

---

# 20. Scheduler

Le Scheduler répartit les tâches sur plusieurs Workers.

```

Queue

↓

Worker 1

Worker 2

Worker 3

Worker 4

```

Les Workers sont indépendants.

---

# 21. Priorités

| Niveau | Utilisation |
|----------|-----------------------------|
| Critical | fermeture d'un coffre |
| High | lecture utilisateur |
| Normal | synchronisation |
| Low | GC |
| Idle | maintenance |

---

# 22. Isolation

Un service ne peut jamais accéder directement aux données d'un autre.

Exemple :

Network

↓

Daemon

↓

Sync

↓

Vault

Jamais :

Network → Storage

---

# 23. Santé des services

Chaque service expose :

```

Health

Ready

Live

Metrics

Version

```

Le Daemon surveille ces indicateurs.

---

# 24. Auto-récupération

En cas de panne d'un service :

```

Service Failed

↓

Isolation

↓

Restart

↓

Reconnect

↓

Resume

```

Les autres services continuent de fonctionner.

---

# 25. Arrêt

Le Daemon effectue un arrêt ordonné.

```

Stop IPC

↓

Stop Sync

↓

Flush Storage

↓

Close Vaults

↓

Stop Network

↓

Shutdown

```

Aucune donnée ne doit être perdue.

---

# 26. Observabilité

Chaque requête reçoit :

- Correlation ID
- Session ID
- User ID
- Vault ID
- Request ID

Ces identifiants sont présents dans tous les logs.

---

# 27. Extension

Les futurs services pourront être ajoutés sans modifier le microkernel.

Exemple :

```

Search Service

Thumbnail Service

Backup Service

Messenger Service

AI Service

```

Le Daemon ne dépend jamais de leur implémentation.

---

# 28. Principes

Le Daemon :

✓ orchestre

✓ sécurise

✓ supervise

✓ distribue

Le Daemon :

✗ ne chiffre pas

✗ ne synchronise pas

✗ ne lit pas les blocs

✗ ne modifie pas les manifests

---

# Conclusion

Le Daemon constitue le cœur opérationnel de SyFi.

En l'organisant comme un microkernel composé de services indépendants, le système gagne en robustesse, en extensibilité et en testabilité.

Toutes les futures applications de l'écosystème pourront s'appuyer sur ce même Daemon, sans duplication de logique métier.