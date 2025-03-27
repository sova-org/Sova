# BuboCore

[![Crates.io](https://img.shields.io/crates/v/bubocore.svg)](https://crates.io/crates/bubocore) [![Documentation](https://docs.rs/bubocore/badge.svg)](https://docs.rs/bubocore) [![Build Status](https://github.com/username/bubocore/workflows/Rust/badge.svg)](https://github.com/username/bubocore/actions)

![](https://raphaelforment.fr/images/bubobubo.jpg)

## Qu'est-ce que BuboCore ?

BuboCore est un séquenceur et un environnement de programmation musicale conçu en Rust. Il est spécialement conçu pour le [live coding](https://livecoding.fr) musical. Il permet aux musiciens et artistes de composer ou d'improviser de la musique de manière réactive, en temps réel, au travers du code. BuboCore est un logiciel libre et open-source, destiné à être utilisé aussi bien par des musiciens que des artistes, des développeurs, des chercheurs, des enseignants, etc. Il se destine à populariser une approche instrumentale, créative et musicale de la programmation informatique, à faire du code un terrain d'expression artistique.

**Note :** l'outil est aujourd'hui encore aux premiers stades de son développement. Utilisez-le à vos risques et périls !

## Fonctionnalités principales

- **Un outil polyglotte** : BuboCore est un terrain d'expérimentation pour la création de langages de programmation musicaux. Son architecture autorise la prise en charge d'un ou de plusieurs langages pouvant être librement combinés au cours d'une même session. Chaque script soumis au moteur est traduit/compilé dans une représentation machine interne puis exécuté par le moteur. BuboCore offre pour les développeurs et les musiciens un outil de travail pour imaginer différents langages dédiés à l'écriture de séquences musicales / séquences d'événements.

- **Réactif et live codable** : tout programme en cours d'exécution peut être modifié à la volée. Les modifications sont prises en compte immédiatement, sans interruption. Plusieurs _scripts_ peuvent être exécutés de manière séquentielle ou simultanée. La configuration du logiciel ou le comportement interne du moteur peut lui aussi être altéré au cours du jeu.

- **Architecture client/serveur** : BuboCore est conçu sur le modèle d'une architecture client/serveur. Ceci permet de jouer seul aussi bien qu'en groupe. Ceci permet aussi de disposer de plusieurs clients distincts, développés indépendamment. Chaque client peut éventuellement ajouter des fonctionnalités spécifiques au-delà des seules capacités offertes par le moteur.

- **Synchronisation Ableton Link et MIDI :** BuboCore peut se synchroniser avec l'écrasante majorité des autres logiciels de création et instrument électroniques : synthétiseurs, _groovebox_, boîtes à rythmes, etc. BuboCore peut aussi bien jouer le rôle d'interface de jeu principale que d'instrument au sein d'un ensembel d'instruments plus large.

- **Multi-protocole** : BuboCore est capable de communiquer avec d'autres logiciels ou instruments via MIDI et OSC. Des protocoles plus spécialisés peuvent également être implémentés sans difficultés au moteur.


## Installation

### Via Cargo

[Cargo](https://doc.rust-lang.org/cargo/) est le gestionnaire de paquets utilisé par le langage Rust. Après son installation, vous pouvez installer BuboCore en exécutant la commande suivante :
```bash
cargo install bubocore
```

### Via un paquet binaire

Dirigez-vous vers la section [Releases](https://github.com/Bubobubobubobubo/BuboCore/releases) pour télécharger le paquet binaire correspondant à votre système d'exploitation. Des instructions d'installation supplémentaires seront détaillées avec chaque _release_.

### Depuis les sources

Pour profiter des fonctionnalités les plus récentes, vous pouvez cloner le dépôt et compiler le logiciel vous-même :

```bash
git clone https://github.com/username/bubocore.git
cd bubocore
cargo build --release
```

## Documentation

### Pour les utilisateurs

Une documentation utilisateur de BuboCore est disponible à l'adresse suivante : [bubocore.livecoding.fr](https://bubocore.livecoding.fr).

### Pour les développeurs

Une documentation technique est disponible à l'adresse suivante : [docs.rs](https://docs.rs/bubocore).

## Communauté

- [Discord]()
- [Github Issues]()

## Comment contribuer ?

Contribuer au développement de BuboCore ne nécessite pas nécessairement de programmer. Vous pouvez aider en signalant des bugs, en proposant des améliorations, en corrigeant des fautes d'orthographe, en traduisant la documentation, etc. Ce dépôt est un espace de travail collaboratif. Pour les développeurs, référez-vous au fichier [CONTRIBUTING.md](/CONTRIBUTING.md).

- **Programmation :** Correction de bugs, ajout de fonctionnalités. Voir [la liste des issues ouvertes]().
- **Documentation :** Amélioration des guides, correction des fautes, traduction du contenu.
- **Tests :** Tester BuboCore, signaler les problèmes rencontrés.
- **Idées :** Proposer de nouvelles fonctionnalités ou améliorations.
- **Communauté :** Répondre aux questions d'autres utilisateurs.

## License

BuboCore est distribué sous licence [GPL-3.0](https://www.gnu.org/licenses/gpl-3.0.html). Une copie de la license est distribuée avec le logiciel : [LICENSE](/LICENSE).

## Remerciements

### Institutions

- Le [Laboratoire LS2N](https://www.ls2n.fr) et l'[Université de Nantes](https://www.univ-nantes.fr)
- L'[Athenor CNCM](https://www.athenor.com) et son directeur, Camel Zekri

### Contributions
