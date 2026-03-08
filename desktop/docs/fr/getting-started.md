# Pour commencer avec Sova

Sova est un séquenceur polyglotte spécialisé pour le live coding. Le code que
vous évaluez produit des événements musicaux — notes MIDI, control changes,
messages OSC — joués sur une timeline commune synchronisée via Ableton Link.
Plusieurs langages, plusieurs musiciens, un seul tempo.

## Lancer l'application

Au lancement de Sova, vous pouvez soit démarrer un **serveur intégré**, soit
vous connecter à un **serveur distant** hébergé par quelqu'un d'autre.

- **Serveur intégré** : Ouvrez le panneau Serveur et cliquez sur Démarrer.
  L'application crée un serveur local et s'y connecte automatiquement. C'est la
  manière la plus simple de commencer en solo.
- **Serveur distant** : Entrez l'adresse et le port dans le panneau Serveur et
  cliquez sur Connecter. Vous rejoindrez une session existante avec d'autres
  musiciens.

Une fois connecté, vous verrez la grille de scène — le cœur de l'interface.

## L'interface en un coup d'œil

L'interface de Sova s'articule autour de panneaux que vous pouvez afficher,
masquer et réorganiser :

- **Grille de scène** — l'espace de travail principal. Les lignes se lisent de
  gauche à droite en colonnes, les frames s'empilent de haut en bas en rangées.
  C'est ici que vous écrivez et organisez votre code.
- **Barre de transport** — lecture/arrêt, tempo et quantum en haut.
- **Panneau Serveur** — paramètres de connexion et état du serveur.
- **Panneau Périphériques** — gestion des ports MIDI, des points d'accès OSC et
  des sorties audio.
- **Panneau Audio** — configuration du moteur audio intégré (Doux).
- **Oscilloscope / Spectre / VU-mètre** — visualisation de la sortie audio en
  temps réel.
- **Panneau Journaux** — événements et messages de débogage.
- **Panneau Chat** — communiquez avec les autres musiciens en session
  multijoueur.
- **Panneau Visuels** — écrivez du code visuel style Hydra, affiché en fond.
- **Panneau Options** — thème de l'éditeur, taille de police et autres
  préférences.
- **Panneau Documentation** — le panneau que vous lisez en ce moment.

Faites un clic droit sur un espace vide de la grille pour afficher ou masquer
les panneaux.

## Produire du son

Pour entendre quelque chose, il faut au moins un périphérique de sortie :

- **Sortie MIDI** : Ouvrez le panneau Périphériques, connectez un port MIDI
  matériel ou créez une sortie MIDI virtuelle. Assignez-la à un slot (1–16).
  Votre code envoie des événements vers un numéro de slot, et le périphérique assigné à ce slot les joue.
- **Moteur audio** : Si le serveur a été démarré avec le support audio, le
  synthétiseur intégré (Doux) est disponible sur un slot. Ouvrez le panneau
  Audio pour le configurer.
- **Sortie OSC** : Créez un point d'accès OSC dans le panneau Périphériques pour
  envoyer des messages vers un logiciel externe (SuperCollider, Max/MSP, etc.).

Consultez l'article **Périphériques** pour tous les détails de configuration.
