---
title: "BuboCore"
type: 'docs'
bookHidden: false
bookToC: false
---

# BuboCore : un séquenceur musical pour le _live coding_

## Qu'est-ce que BuboCore ?

{{% columns ratio="4:1" %}}

BuboCore est un logiciel protéiforme.
Il peut aussi bien être décrit comme un environnement de programmation créative que comme un séquenceur musical.
C'est un outil d'expérimentation artistique aussi bien qu'un outil conçu pour réfléchir à la conception de langages
musicaux pour l'improvisation et la performance musicale. BuboCore est un outil visant à faciliter la pratique du [_live coding_](https://livecoding.fr/). Ce logiciel cherche à encourager tout musicien qui s'en saisit à adopter une approche performative de la programmation.
Son fonctionnement incite à percevoir l'ordinateur comme un instrument musical, à prêter attention aux aspects créatifs et poétiques de l'expression au travers du code.

<--->

![](bubobubo.jpg)


{{% /columns %}} 

{{% columns %}}

## Quel est son principe de fonctionnement ?

BuboCore est basé sur le même principe de fonctionnement que le séquenceur à pas d'une boîte à rythmes.
Ce modèle fondamental est ici repensé et adapté pour l'expérience particulière de la programmation à la volée.
Chaque pas composant une séquence musicale est ici représenté sous la forme d'un court programme informatique : un script.
Chaque script est d'une longueur et d'une complexité arbitraire.
Il peut avoir, lors de son exécution, différents effets : émission de notes, de messages,
modification de paramètres, de l'état du séquenceur et/ou du programme, etc.
Les _scripts_ sont libres d'interagir avec l'ensemble de l'environnement.
L'environnement se compose quant à lui de différentes connexions à des logiciels et ou des machines externes.
Plusieurs séquences de _scripts_ peuvent être jouées de concert, interrompues et/ou reprogrammées à la volée !
Les _scripts_ sont exécutés en rythme, avec une précision temporelle métronomique.
Le musicien possède un contrôle un contrôle algorithmique complet aussi bien sur la définition des séquences que
sur leur exécution ou sur le comportement du séquenceur lui-même. BuboCore est donc, pour résumer, un outil de composition
et d'improvisation musicale permettant l'écriture dans le temps de séquences d'évènements.

<--->

## À qui s'adresse BuboCore ?

BuboCore a été pensé dès son origine pour accompagner l'apprentissage de la programmation et/ou de l'informatique musicale.
Le logiciel est donc accessible pour tout musicien débutant, pour tout amateur et tout curieux. Aucun prérequis
technique ou musical n'est nécessaire pour commencer à l'utiliser. Toute la complexité naît de la maîtrise graduelle acquise
par le jeu et l'expérimentation avec le logiciel, de la maîtrise des concepts élémentaires aux techniques de programmation 
les plus avancées. BuboCore est un outil qui cherche à présenter de nouvelles manières de penser et de concevoir, au travers du
code, l'expression musicale.

Ce logiciel intéressera aussi des musiciens plus expérimentés. Ils trouveront dans les ressources offertes par BuboCore un ensemble
d'outils et de techniques permettant de contrôler avec précision leurs machines, leurs synthétiseurs et leurs différents outils 
de génération sonore/visuelle, etc. BuboCore peut servir à faciliter la préparation de performances complexes et/ou aider
à la formalisation de certains techniques et/ou pensées musicales : composition algorithmique, générative, stochastique, etc.

{{% /columns %}} 

## Quels langages de programmation sont supportés par BuboCore ?

BuboCore est conçu pour héberger différents langages de programmation spécialisés dans la description d'événements musicaux.
Chaque _script_ d'une séquence peut être programmé à l'aide d'un langage de programmation choisi (et potentiellement créé !)
par le musicien.
Certains langages se spécialisent dans l'écriture de notes, d'autres dans la description d'évènements ou de procédés plus abstraits.
Pouvoir choisir ou construire différents langages permet d'explorer librement différentes manières de programmer et de penser la musique.
Cela permet aussi de trouver le langage et les abstractions les plus adaptées à un style de jeu, à une manière de faire, etc.
Différents langages peuvent être ajoutés à condition que ceux-ci puissent être compilés/interprétés dans la représentation intermédiaire utilisée par le  moteur événémentiel interne de BuboCore. Au fondement de BuboCore se trouve un langage intermédiaire générique et puissant,
proche du langage machine, permettant de décrire de manière abstraite des programmes musicaux sous une forme impérative.


## Quel rôle joue BuboCore dans un environnement de création musicale ?

BuboCore n'émet aucun son, il s'agit d'un logiciel intermédiaire. Il est pensé pour être utilisé en conjonction
avec d'autres logiciels de création musicale, synthétiseurs, boîtes à rythmes, logiciels et langages de traitement du signal, etc.
L'outil est entièrement tourné vers la communication inter-logicielle et la synchronisation/synergie avec d'autres outils ou musiciens.
BuboCore peut émettre ou recevoir des messages MIDI et OSC. Il peut être synchronisé au travers du protocole Ableton Link ou d'une horloge MIDI.
Le logiciel peut aussi servir de contrôleur central et de métronome pour d'autres logiciels ou machines.


![](bubocore.png)

{{% columns ratio="1:2" %}}

## À propos


![](athenor_logo.jpg)

<--->

##

_BuboCore_ est un logiciel de création musicale conçu dans le cadre d'un
projet de recherche soutenu par l'[Athénor CNCM](https://www.athenor.com/) de Saint-Nazaire et par le [laboratoire LS2N](https://www.ls2n.fr/) de l'université de Nantes.
Ce logiciel est disponible en libre accès et sous licence _open source_.
Nous acceptons les contributions de la communauté et nous vous invitons
à expérimenter librement avec l'outil afin de travailler à son amélioration.

![](ls2n_logo.jpg)


{{% /columns %}}
