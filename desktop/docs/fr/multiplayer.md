# Multijoueur

Se connecter a un serveur, voir ou sont les autres dans la grille, editer du
code en meme temps, chatter, jouer. Les sessions Sova sont multijoueur par
defaut.

## Heberger une session

Deux options.

Le serveur integre : ouvrez le panneau Serveur, cliquez Demarrer. L'application
lance un serveur en interne et s'y connecte. Les autres joueurs se connectent a
votre IP et votre port.

Le serveur autonome : lancez `sova-server` en ligne de commande.

```
sova-server -p 8080
```

Preferable pour un hebergement dedie ou une machine sans ecran. Meme serveur,
pas d'interface graphique.

Le serveur possede la scene, l'horloge et le routage des peripheriques. Les
clients sont legers : ils envoient des modifications et recoivent l'etat.

## Rejoindre

Ouvrez le panneau Serveur. Entrez l'adresse, le port et un nom d'utilisateur.
Cliquez Connecter.

Vous recevez la scene complete, la configuration des peripheriques et l'etat de
l'horloge immediatement. Le transport se synchronise via Ableton Link -- le beat
est deja cale quand la grille s'affiche.

Les noms d'utilisateur doivent etre uniques dans la session. Si le votre est
pris, choisissez-en un autre.

## Ce qui se synchronise

Tout ce qui touche a la scene passe par le serveur :

- Structure de la scene : lignes, frames, durees, repetitions, scripts
- Etat du transport : lecture, arret, tempo, quantum
- Assignation des peripheriques : quel slot correspond a quelle sortie
- Evaluation du code : quand vous evaluez une frame, le serveur compile et planifie

Quand vous vous deconnectez puis reconnectez, vous recevez la scene en cours.
Pas de memoire d'etat local.

## Ce qui ne se synchronise pas

- La disposition de vos panneaux et vos preferences d'editeur restent locales
- Les connexions MIDI et OSC sont propres a chaque machine (chaque joueur
  configure ses sorties dans le panneau **Peripheriques**)
- Les scripts visuels (Hydra) tournent cote client

## Edition collaborative

La position de chaque joueur dans la grille est visible par tous. Des
indicateurs colores apparaissent sur les cellules que les autres consultent ou
editent.

Quand quelqu'un ouvre l'editeur d'une frame, la grille le signale. Ca donne une
vision naturelle de qui travaille ou.

Pas de verrouillage. Deux joueurs peuvent editer des frames differentes en meme
temps sans conflit. Si deux joueurs editent la meme frame, la derniere
evaluation l'emporte.

## Chat

Le panneau Chat envoie des messages texte a toute la session. Pratique pour
coordonner les transitions en plein set : "je lache la basse au prochain
quantum", "je passe en noise sur la ligne 3".

## Conseils pour le jeu collectif

Revendiquez vos propres lignes. Si vous restez sur les lignes 1-2 et votre
partenaire sur 3-4, vous evitez de vous marcher dessus.

Convenez des slots de peripheriques avant de commencer. Le slot 1 pour le
synthe, le slot 3 pour les drums, ce qui vous convient. Si quelqu'un reassigne
un slot partage en plein set, tout ce qui y transite change.

Ableton Link maintient le beat serre entre les machines du meme reseau. Les
changements de tempo se propagent a toutes les applications Link, pas seulement
aux clients Sova.

Utilisez le reglage de quantum pour coordonner les transitions. Un quantum de
4 beats fait atterrir les changements sur la mesure suivante. Un quantum de
8 beats donne plus de marge.
