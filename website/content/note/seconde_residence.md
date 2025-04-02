+++
date = 2024-02-05
draft = false
title = 'Résidence (24-28 mars 2025)'
weight = 90 
[params]
  author = 'Raphaël Forment'
+++

# Résidence de développement

## Introduction

Du 24 au 28 mars 2025 s'est tenue grâce au soutien financier de l'[Athenor CNCM](https://athenor.com) une semaine de résidence dédiée au développement logiciel de _BuboCore_. Cette résidence a été accueillie conjointement par le laboratoire [LS2N](https://ls2n.fr) puis par l'[IUT de Nantes](https://iutnantes.univ-nantes.fr/). L'objectif de cette session de travail fut d'asseoir définitivement les bases de l'architecture logicielle et de travailler à l'obtention d'une première version utilisable. C'est au cours de cette semaine que _BuboCore_ fut pour la première fois en mesure d'émettre ses premières notes et de jouer ses premiers _scripts_ ! C'est au cours de cette même semaine qu'un premier langage minimal pour BuboCore a pu être développé par [Loïg Jezequel](http://localhost:1313/docs/apropos/#lo%c3%afg-jezequel) ou qu'une architecture client/serveur a pu être mise en place par [Tanguy Dubois](http://localhost:1313/docs/apropos/#tanguy-dubois). [Raphaël Forment](/docs/apropos/#raphael-forment) s'est chargé de l'implémentation MIDI et de la mise en place du premier client : _bubocoretui_ (voir Figure 2). Le site internet que vous consultez actuellement est un autre des produits immédiats de cette résidence.

{{< center >}}
{{< image-legend src="/mars2025/ecole_centrale_nantes.jpg" alt="Ecole centrale de Nantes" caption="Locaux du laboratoire LS2N de l'École Centrale de Nantes." >}}
{{< /center >}}

Le projet s'est rapidement solidifié autour de deux entités logicielles aux rôles bien distincts :

- **bubocore** : un serveur d'exécution et de distribution de l'information musicale entre clients, hébergeant un premier compilateur pour un langage haut-niveau (*BaLi* pour *Basic Lisp*) et l'ensemble du code essentiel au fonctionnement de l'application.
- **bubocoretui** : un modèle d'interface utilisateur, possédant un éditeur de texte ainsi que différents outils de contrôle pour le serveur. Ce logiciel se destine à faciliter le débogage et le test de l'application (voir Figure 2).

{{< center >}}
{{< image-legend src="/mars2025/bubocoretui.png" alt="Interface du TUI bubocoretui" caption="Interface de l'utilitaire _bubocoretui_, premier client développé pour _BuboCore_." >}}
{{< /center >}}

Les principaux problèmes laissés en suspens par la précédente session de travail semblent à ce point être résolus et les performances actuelles de l'application sont satisfaisantes. Il faudra toutefois attendre les premières _jams_ collaboratives pour disposer d'un véritable retour utilisateur ! Le développement s'oriente donc désormais vers le fait de _rendre possible_ cette première _jam_ entre deux villes : Lyon (Rémi Georges et Raphaël Forment) et Nantes (Loïg Jezequel et Tanguy Dubois). Quelques programmes de tests sont d'ores et déjà testables sur le dépôt logiciel du projet -- et ils tournent !.

## Conclusions

Le projet se rapproche doucement d'une première version collaborative utilisable. Celle-ci permettra la connexion de plusieurs musiciens, l'édition et la soumission de _scripts_ pour exécution, le contrôle de l'ensemble des paramètres du serveur. La structure du projet permet désormais d'envisager l'ajout de fonctionnalités essentielles pour les prochaines étapes de travail :
- support du protocole _OSC_ ([Open Sound Control](https://en.wikipedia.org/wiki/Open_Sound_Control)) pour le contrôle de logiciels externes.
- suite du développement de _Basic Lisp_ pour réaliser des opérations plus complexes.
- ajouter / supprimer / mettre à jour des _scripts_ de manière collaborative.
- choix du langage de _script_ à utiliser pour chaque pas.
- sauvegarde et chargement de sessions utilisateur sous la forme de fichiers.
