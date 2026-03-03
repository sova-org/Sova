# Multijoueur

Sova est conçu pour le live coding collaboratif. Plusieurs musiciens peuvent se
connecter au même serveur, voir le code des autres en temps réel et jouer
ensemble sur une scène partagée.

## Démarrer un serveur

Il y a deux façons d'héberger une session :

- **Serveur intégré** : Lancez l'application Sova, ouvrez le panneau Serveur et
  cliquez sur Démarrer. L'application exécute un serveur en interne et s'y
  connecte. Les autres musiciens peuvent se connecter à l'adresse IP et au port
  de votre machine.
- **Serveur autonome** : Exécutez `sova-server` depuis la ligne de commande avec
  un numéro de port. C'est utile pour un hébergement dédié sur une machine qui
  n'a pas besoin d'interface graphique.

Le serveur gère la scène, l'horloge et toutes les connexions de périphériques.
Les clients sont légers — ils envoient des modifications et reçoivent des mises
à jour.

## Se connecter

Pour rejoindre une session :

1. Ouvrez le panneau Serveur.
2. Entrez l'adresse IP et le port du serveur.
3. Choisissez un nom d'utilisateur (il doit être unique dans la session).
4. Cliquez sur Connecter.

Une fois connecté, vous recevez la scène complète, la configuration des
périphériques et l'état de l'horloge. Vous êtes immédiatement synchronisé avec
tout le monde.

Si le nom d'utilisateur est déjà pris ou si la connexion est refusée, vous
verrez un message d'erreur. Choisissez un autre nom et réessayez.

## Édition collaborative

Quand plusieurs musiciens sont connectés :

- Vous pouvez voir où se trouve le curseur de chaque musicien dans la grille.
  Chaque musicien a un indicateur distinct sur la cellule qu'il consulte ou
  édite.
- Quand quelqu'un commence à éditer une case (ouvre l'éditeur de code), les
  autres musiciens voient que la case est en cours d'édition. Cela aide à éviter
  les modifications conflictuelles.
- Tous les changements de scène — ajout de lignes, modification de cases,
  changement de durées — sont diffusés à chaque client connecté en temps réel.

Il n'y a pas de verrouillage : deux musiciens peuvent éditer des cases
différentes simultanément sans conflit. Si deux musiciens éditent la même case,
la dernière évaluation l'emporte.

## Chat

Le panneau Chat vous permet d'envoyer des messages texte à tous les participants
de la session. Ouvrez-le depuis le menu de panneau ou le menu contextuel de la
grille. Les messages affichent le nom de l'expéditeur.

## Synchronisation de la scène

Le serveur est la source de vérité. Quand vous évaluez du code, ajoutez une
case ou changez une propriété, votre modification est envoyée au serveur, qui
l'applique et diffuse le résultat à tous les clients. Cela signifie :

- Tout le monde voit toujours le même état de la scène.
- Si vous vous déconnectez puis vous reconnectez, vous obtenez la scène
  actuelle, pas votre dernier état local.
- L'horloge du serveur (via Ableton Link) maintient tout le monde aligné
  temporellement.

## Astuces

- Convenez des assignations de slots avec vos collaborateurs. Si le musicien A
  utilise le slot 1 pour le synthé et que le musicien B réassigne le slot 1 à la
  batterie, la confusion est garantie.
- Utilisez des lignes différentes pour différents musiciens afin d'éviter de
  marcher sur le code des autres.
- Le chat est pratique pour coordonner les transitions — « je lâche la basse au
  prochain quantum » — sans interrompre le flux de la performance.
- La synchronisation Ableton Link fonctionne à travers le réseau, donc même si
  les musiciens sont sur des machines différentes, le beat reste verrouillé.
