---
title: "Sova"
type: 'docs'
bookHidden: false
bookToC: false
---

# Sova : un séquenceur musical pour le _live coding_

## À propos du projet

{{% columns ratio="1:2" %}}

{{< center >}}
![](logos/athenor_logo.jpg)
{{< /center >}}

<--->

_Sova_ est un logiciel de création musicale conçu dans le cadre d'un
projet de recherche soutenu par l'[Athénor CNCM](https://www.athenor.com/) de Saint-Nazaire et par le [laboratoire LS2N](https://www.ls2n.fr/) de l'université de Nantes.
Ce logiciel est disponible en libre accès et sous licence _open source_. Il est développé par une petite équipe de développeurs et de contributeurs volontaires.

![](/logos/ls2n_logo.jpg)

{{% /columns %}}

## Qu'est-ce que Sova ?

{{% columns ratio="4:1" %}}

Sova est un logiciel protéiforme. Il peut être décrit comme un environnement de programmation créative et comme un séquenceur musical. C'est un outil d'expérimentation artistique, conçu pour accompagner la réflexion autour de la conception de langages musicaux pour l'improvisation et la performance musicale. Sova est un outil visant à faciliter [la pratique du live coding](https://livecoding.fr). Ce logiciel cherche à encourager tout musicien qui s'en saisit à adopter une approche performative de la programmation. Son fonctionnement incite à percevoir l'ordinateur comme un instrument musical, à prêter attention aux aspects créatifs et poétiques de l'expression au travers du code. Il propose une expérience immédiate, ludique et incarnée de la programmation musicale.

<--->

![](bubobubo.jpg)


{{% /columns %}} 

{{% columns %}}

## Quel est son principe de fonctionnement ?



Sova est basé sur le même principe de fonctionnement que les séquenceurs à pas d'une boîte à rythme traditionnelle. Ce modèle est ici adapté pour se plier à un mode de jeu nouveau : celui de la [programmation à la volée](https://livecoding.fr). Chacun des pas qui composent une séquence musicale sont représentés sous la forme de courts programmes informatiques, des _scripts_. Chaque _script_ est d'une longueur et d'une complexité arbitraire. Il peut avoir différents effets lors de son exécution : émission de notes, de messages, modification de paramètres, de l'état du séquenceur et/ou du programme, etc. Les scripts sont libres d'interagir avec l'ensemble de l'environnement (voir Figure 1). 

{{< center >}}
{{< image-legend src="scene_demo.svg" alt="Démonstration de la structure d'une scène" 
caption="Structure imbriquée d'une scène Sova." >}}
{{< /center >}}

L'environnement du séquenceur se compose des différentes connexions à des logiciels et/ou machines externes. Plusieurs séquences de _scripts_ peuvent être jouées de concert, interrompues et/ou reprogrammées à la volée ! Les scripts sont exécutés en rythme, avec une précision temporelle métronomique. Le musicien possède un contrôle algorithmique complet sur la définition des séquences autant que sur leur exécution ou sur le comportement du séquenceur. L'ensemble des scripts formant une session de jeu sont disponibles pour l'ensemble des musiciens connectés à une même session.

<--->

## À qui s'adresse Sova ?

Sova a été pensé pour accompagner l'apprentissage de la programmation et/ou de l'informatique musicale. Le logiciel est donc accessible pour tout musicien débutant. Aucun prérequis de nature technico-musicale n'est nécessaire pour s'en saisir. Toute la complexité naît de la maîtrise graduelle de l'outil que le musicien acquiert par l'expérimentation et par le jeu. L'utilisation de Sova commence par l'apprentissage des notions musicales et techniques les plus élémentaires : le solfège propre au _live coding_. L'apprentissage s'étend ensuite vers la maîtrise de techniques de programmation/composition plus avancées. Les utilisateurs les plus investis pourront aller jusqu'à modifier l'outil lui-même. Ils possèderont ainsi une maîtrise complète de l'instrument et le feront évoluer avec eux. L'outil est conçu pour être intuitif. Il n'expose que graduellement la complexité de son fonctionnement, toujours à l'initiative du musicien.

Ce logiciel intéressera également des musiciens et artistes plus expérimentés. Ils trouveront dans Sova un outil permettant le contrôle et la synchronisation précise de leurs différentes machines, synthétiseurs, logiciels de génération sonore/ visuelle. Sova est tout à la fois :
- un environnement de programmation et de prototypage extensible, _open source_ et multi-langage.
- un séquenceur musical collaboratif (multi-client) et temps réel.
- un instrument musical algorithmique et réactif.

Sova peut servir à préparer des performances musicales complexes. Il peut aussi aider le musicien à formaliser tout en improvisant certaines techniques de jeu et/ou manières de penser l'écriture et la performance musicale : composition algorithmique, générative stochastique, aléatoire, etc.

{{% /columns %}} 

{{< image-legend src="first_line.jpg" alt="Première séquence Sova" caption="Première séquence musicale compilée avec Sova (mars 2025). À gauche : programme brut, à droite : messages émis." >}}


## Comment interagir avec Sova ?

Sova repose sur une architecture client/serveur. Le serveur coordonne les différents clients utilisés par les musiciens. Il organise l'exécution rythmique et synchrone du code, se connecte aux périphériques externes et aux logiciels qui composent l'environnement. Le serveur peut être exécuté sur une machine dédiée ou sur l'ordinateur de l'un des musiciens utilisateurs. Le serveur est contrôlé conjointement par l'ensemble des clients connectés. Chaque client prend pour les musiciens la forme d'une interface graphique dédiée (voir Figure 3). Les clients permettent de programmer manuellement des séquences, de les jouer, de les modifier, de les arrêter, de les sauvegarder, etc. Les clients peuvent être exécutés sur la même machine que le serveur ou bien à distance, sur une machine distante capable de se connecter au travers du réseau. La connexion entre client et serveur s'effectue au travers du protocole TCP. Chaque communication est sérialisée/désérialisée au format JSON, permettant à Sova d'être facilement extensible et modularisé.

{{< image-legend src="bubocore_client_splash.png" alt="Exemple de client Sova: sovatui" caption="Exemple d'un client Sova utilisé pour les tests : _sovatui_. Sur l'image, vue de la page de connexion au serveur." >}}


## Quels langages de programmation supporte Sova ?

Sova est conçu pour supporter différents langages de programmation construits _ad hoc_ pour le logiciel. Ces langages sont spécialisés dans la description d'événements ou de séquences musicales. Chaque _script_ peut être programmé, au besoin, à l'aide d'un langage de programmation différent. Certains langages se spécialiseront naturellement dans l'écriture de séquences mélodico-harmoniques, d'autres dans la description de rythmes, d'évènements ou de procédés plus abstraits. Le serveur Sova prend en charge la transmission de ces _scripts_, écrits dans des langages de haut-niveau, vers une représentation machine interne, proche de l'assembleur. Si le protocole de communication avec le serveur est respecté, des scripts écrits dans des langages très différents peuvent co-exister et être exécutés sans problème sur le serveur. Différents langages peuvent être ajoutés à condition que ceux-ci puissent être compilés/interprétés dans la représentation intermédiaire utilisée par le moteur événémentiel interne de Sova. Au fondement de Sova se trouve un langage générique et puissant permettant de décrire de manière abstraite des programmes musicaux sous une forme synchrone/impérative. 


{{< image-legend src="test_export.svg" alt="Architecture client-serveur" caption="Architecture client/serveur, plusieurs langages de _script_ sont interprétés vers une seule et même représentation interne." >}}

{{% columns %}}

```lisp
// Envoi d'une note
(@ 0 (n c 90 1))
```
**Exemple 1a :** un script utilisateur (langage *BaLi* pour _Basic Lisp_).

<--->

```rust
let note: Program = vec![Instruction::Effect(
    Event::MidiNote(
        60.into(),
        90.into(),
        1.into(),
        TimeSpan::Beats(1.0).into(),
        midi_name.clone().into(),
    ),
    TimeSpan::Micros(1_000_000).into(),
)];
```
**Exemple 1b :** le même programme en notation interne (_Rust_).

{{% /columns %}}

Pouvoir construire différents langages et choisir lequel employer en fonction de la situation, du dispositif et/ou du projet permet d'explorer librement différentes manières de programmer et de penser la musique. Chaque langage de programmation induit également un rapport différent du musicien à l'instrument. Les musiciens peuvent choisir les abstractions les plus adaptées à leur style de jeu, à leur manière de faire et de collaborer (jeu multi-client). Il n'est pas nécessaire pour les développeurs de maîtriser le langage Rust pour proposer de nouveaux langages. Le serveur possède une interface permettant de soumettre un programme sérialisé au format JSON, qui sera ensuite traduit en langage machine et exécuté par Sova.


## Quel rôle joue Sova dans un environnement de création musicale ?

Sova est un outil _middleware_ : il n'émet aucun son. Le logiciel occupe une position d'intermédiaire et de médiateur dans un environnement de création musicale. Il est pensé pour être utilisé en conjonction d'autres logiciels de création musicale, synthétiseurs, boîtes à rythmes, outils de traitement du signal, etc. L'outil est entièrement tourné vers la communication inter-logicielle et la synchronisation. Sova peut émettre ou recevoir des messages MIDI et OSC. Il peut être synchronisé au travers du protocole Ableton Link mais aussi, au besoin, d'une horloge MIDI. Le logiciel peut aussi servir de contrôleur central et de métronome pour d'autres logiciels ou machines.

{{< video src="first_line.mp4">}}



