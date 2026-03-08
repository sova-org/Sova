# Multijoueur

Se connecter à un serveur, voir où sont les autres dans la grille, éditer du
code en même temps, chatter, jouer. Les sessions Sova sont multijoueur par
défaut.

## Héberger une session

Deux options.

Le serveur intégré : ouvrez le panneau Serveur, cliquez Démarrer. L'application
lance un serveur en interne et s'y connecte. Les autres joueurs se connectent à
votre IP et votre port.

Le serveur autonome : lancez `sova-server` en ligne de commande.

```
sova-server -p 8080
```

Préférable pour un hébergement dédié ou une machine sans écran. Même serveur,
pas d'interface graphique.

Le serveur possède la scène, l'horloge et le routage des périphériques. Les
clients sont légers : ils envoient des modifications et reçoivent l'état.

## Rejoindre

Ouvrez le panneau Serveur. Entrez l'adresse, le port et un nom d'utilisateur.
Cliquez Connecter.

Vous recevez la scène complète, la configuration des périphériques et l'état de
l'horloge immédiatement. Le transport se synchronise via Ableton Link -- le beat
est déjà calé quand la grille s'affiche.

Les noms d'utilisateur doivent être uniques dans la session. Si le vôtre est
pris, choisissez-en un autre.

## Ce qui se synchronise

Tout ce qui touche à la scène passe par le serveur :

- Structure de la scène : lignes, frames, durées, répétitions, scripts
- État du transport : lecture, arrêt, tempo, quantum
- Assignation des périphériques : quel slot correspond à quelle sortie
- Évaluation du code : quand vous évaluez une frame, le serveur compile et planifie

Quand vous vous déconnectez puis reconnectez, vous recevez la scène en cours.
Pas de mémoire d'état local.

## Ce qui ne se synchronise pas

- La disposition de vos panneaux et vos préférences d'éditeur restent locales
- Les connexions MIDI et OSC sont propres à chaque machine (chaque joueur
  configure ses sorties dans le panneau **Périphériques**)
- Les scripts visuels (Hydra) tournent côté client

## Édition collaborative

La position de chaque joueur dans la grille est visible par tous. Des
indicateurs colorés apparaissent sur les cellules que les autres consultent ou
éditent.

Quand quelqu'un ouvre l'éditeur d'une frame, la grille le signale. Ça donne une
vision naturelle de qui travaille où.

Pas de verrouillage. Deux joueurs peuvent éditer des frames différentes en même
temps sans conflit. Si deux joueurs éditent la même frame, la dernière
évaluation l'emporte.

## Chat

Le panneau Chat envoie des messages texte à toute la session. Pratique pour
coordonner les transitions en plein set : "je lâche la basse au prochain
quantum", "je passe en noise sur la ligne 3".

## Conseils pour le jeu collectif

Revendiquez vos propres lignes. Si vous restez sur les lignes 1-2 et votre
partenaire sur 3-4, vous évitez de vous marcher dessus.

Convenez des slots de périphériques avant de commencer. Le slot 1 pour le
synthé, le slot 3 pour les drums, ce qui vous convient. Si quelqu'un réassigne
un slot partagé en plein set, tout ce qui y transite change.

Ableton Link maintient le beat serré entre les machines du même réseau. Les
changements de tempo se propagent à toutes les applications Link, pas seulement
aux clients Sova.

Utilisez le réglage de quantum pour coordonner les transitions. Un quantum de
4 beats fait atterrir les changements sur la mesure suivante. Un quantum de
8 beats donne plus de marge.
