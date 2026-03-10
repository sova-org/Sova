# Multijoueur

Sova est multijoueur par défaut. On se connecte à un serveur, on voit les
positions des autres dans la grille, on édite du code simultanément et on
communique via le chat intégré. Le live coding est une pratique de partage : le
code est visible, les idées circulent, la musique se construit collectivement.

## Héberger une session

Deux options.

Le serveur intégré : ouvrez le panneau Serveur et cliquez sur Démarrer.
L'application lance un serveur en interne et s'y connecte. Les autres musiciens
se connectent à votre adresse IP et votre port.

Le serveur autonome : lancez `sova-server` en ligne de commande.

```
sova-server -p 8080
```

Cette option convient mieux à un hébergement dédié ou à une machine sans écran.
Le serveur est le même, sans interface graphique.

Le serveur possède la scène, l'horloge et le routage des périphériques. Les
clients sont légers : ils envoient des modifications et reçoivent l'état.

## Rejoindre

Ouvrez le panneau Serveur. Entrez l'adresse, le port et un nom d'utilisateur.
Cliquez sur Connecter.

Vous recevez la scène complète, la configuration des périphériques et l'état de
l'horloge immédiatement. Le transport se synchronise via Ableton Link — le beat
est déjà synchronisé lorsque la grille s'affiche.

Les noms d'utilisateur doivent être uniques dans la session. Si le vôtre est
déjà pris, choisissez-en un autre.

## Ce qui se synchronise

Tout ce qui touche à la scène passe par le serveur :

- Structure de la scène : lignes, frames, durées, répétitions, scripts
- État du transport : lecture, arrêt, tempo, quantum
- Assignation des périphériques : quel slot correspond à quelle sortie
- Évaluation du code : lorsque vous évaluez une frame, le serveur compile et
  planifie

Lorsque vous vous déconnectez puis reconnectez, vous recevez la scène en cours.
Aucun état local n'est conservé.

## Ce qui ne se synchronise pas

- La disposition de vos panneaux et vos préférences d'éditeur restent locales
- Les connexions MIDI et OSC sont propres à chaque machine (chaque musicien
  configure ses sorties dans le panneau **Périphériques**)
- Les scripts visuels (Hydra) tournent côté client

## Édition collaborative

La position de chaque musicien dans la grille est visible par tous. Des
indicateurs colorés apparaissent sur les cellules que les autres consultent ou
éditent.

Lorsqu'un musicien ouvre l'éditeur d'une frame, la grille le signale. Cela
offre une vision claire de qui travaille où.

Aucun verrouillage. Deux musiciens peuvent éditer des frames différentes en même
temps sans conflit. Si deux musiciens éditent la même frame, la dernière
évaluation l'emporte.

## Chat

Le panneau Chat envoie des messages texte à toute la session. Utile pour
coordonner les transitions en pleine performance : « j'arrête la basse au
prochain quantum », « je passe en bruit sur la ligne 3 ».

## Conseils pour le jeu collectif

Revendiquez vos propres lignes. Si vous restez sur les lignes 1–2 et votre
partenaire sur 3–4, vous évitez les conflits d'édition.

Convenez des slots de périphériques avant de commencer. Le slot 1 pour le
synthétiseur, le slot 3 pour la batterie — selon vos besoins. Si un musicien
réassigne un slot partagé en pleine performance, tout ce qui y transite change.

Ableton Link maintient le beat synchronisé entre les machines du même réseau.
Les changements de tempo se propagent à toutes les applications Link, pas
seulement aux clients Sova.

Utilisez le réglage de quantum pour coordonner les transitions. Un quantum de
4 beats fait atterrir les changements sur la mesure suivante. Un quantum de
8 beats offre davantage de marge.
