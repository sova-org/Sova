# Pour commencer avec Sova

Sova est un séquenceur de live coding polyglotte pour l'improvisation musicale
en temps réel. Vous écrivez du code qui génère des événements musicaux — notes
MIDI, changements de contrôle, messages OSC — et Sova les joue sur une timeline
partagée synchronisée via Ableton Link. Plusieurs langages, plusieurs musiciens,
un seul tempo.

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
  gauche à droite en colonnes, les cases s'empilent de haut en bas en rangées.
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
- **Panneau Options** — thème de l'éditeur, taille de police et autres
  préférences.
- **Panneau Documentation** — le panneau que vous lisez en ce moment.

Faites un clic droit sur un espace vide de la grille pour afficher ou masquer
les panneaux.

## Écrire votre première séquence

1. Connectez-vous à un serveur (local ou distant).
2. Cliquez sur une case dans la grille de scène. Chaque case contient un script.
3. Double-cliquez sur la case (ou appuyez sur Entrée) pour ouvrir l'éditeur.
4. Choisissez un langage dans le menu déroulant — Bob, Boinx, Cagire ou BaLi.
5. Tapez un court programme. Par exemple, en Bob :
   ```
   >> [note: 60 vel: 100 dur: 0.5]
   ```
6. Appuyez sur Cmd+Entrée (ou Ctrl+Entrée) pour évaluer.

La case commence à produire des événements immédiatement. Vous verrez la sortie
dans le panneau Journaux, et si un périphérique MIDI est connecté, vous
entendrez du son.

## Produire du son

Pour entendre quelque chose, il faut au moins un périphérique de sortie :

- **Sortie MIDI** : Ouvrez le panneau Périphériques, connectez un port MIDI
  matériel ou créez une sortie MIDI virtuelle. Assignez-la à un slot (1–16).
  Votre code envoie des événements vers un numéro de slot, et le périphérique
  dans ce slot les joue.
- **Moteur audio** : Si le serveur a été démarré avec le support audio, le
  synthétiseur intégré (Doux) est disponible sur un slot. Ouvrez le panneau
  Audio pour le configurer.
- **Sortie OSC** : Créez un point d'accès OSC dans le panneau Périphériques pour
  envoyer des messages vers un logiciel externe (SuperCollider, Max/MSP, etc.).

Consultez l'article **Périphériques** pour tous les détails de configuration.

## Pour aller plus loin

Maintenant que vous savez écrire et écouter une séquence de base, explorez le
reste de la documentation :

- **La Scène** — comprendre les lignes, les cases et leur articulation.
- **La Grille** — raccourcis clavier et flux de travail.
- **Langages** — choisir le langage adapté à votre style.
- **Timing** — tempo, battements, Ableton Link et synchronisation.
- **Événements** — notes MIDI, CC, OSC et routage.
- **Variables** — partager des données entre scripts avec des variables scopées.
- **Multijoueur** — jouer avec d'autres musiciens sur le réseau.
- **Moteur audio** — utiliser le synthétiseur intégré.
