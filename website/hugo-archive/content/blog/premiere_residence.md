+++
date = 2024-02-05
title = 'Campement Scientifique (3-6 février 2025)'
type = 'posts'
weight = 100
+++

# Campement scientifique

Le 5 et 6 février dernier se tenait à l'[Athenor CNCM](https://athenor.com) l'évènement du [campement scientifique](https://athenor.com/les-rendez-vous/2024-25/campement-scientifique-essai), consacré à la mise en valeur des initiatives de recherche-création dans le domaine de la création musicale contemporaine :

> Le Campement scientifique s'inscrit dans un axe de recherche et de création arts-sciences qu'Athénor développe depuis plus de 10 ans en partenariat avec l’université de Nantes et notamment le Laboratoire de Mathématiques Jean-Leray et le Laboratoire des sciences du numérique LS2N. Cette association permet de mettre en place une articulation entre la recherche et la création artistique. Le Campement scientifique est l'occasion de présenter, sous forme de colloque à Saint-Nazaire, le résultat de ces recherches et ce en lien avec la Nuit Blanche des Chercheur·es qui accueille chaque année un projet d’Athénor dans sa programmation et aura lieu le 6 février à Nantes.

Cet événement était organisé autour de différentes conférences, performances artistiques et ateliers d'initiation. Elle était aussi à mettre en lien avec la [Nuit Blanche des Chercheurs](https://nbc.univ-nantes.fr/) organisée par l'Université de Nantes, et dont l'[Athenor CNCM](https://athenor.com) est l'un des partenaires. Raphaël est intervenu à cette occasion pour une performance musicale donnée à Polytech Nantes (campus de Saint-Nazaire), suivie d'une conférence portant sur le projet _Sova_ imaginé quelques semaines auparavant.

## Introduction du projet

Raphaël Forment (_BuboBubo_) et Rémi Georges (_Ralt144mi_), musiciens _live coders_, ont été invités à l'occasion du campement scientifique à présenter et à mettre en place le projet du logiciel _Sova_. Ce logiciel se destine aussi bien à la pédagogie du [live coding](https://livecoding.fr) qu'à résoudre certaines contraintes liées à ce type de performances musicales. [Loïg Jezequel](/docs/apropos/#lo%c3%afg-jezequel), enseignant-chercheur au sein du laboratoire [LS2N](https://ls2n.fr), s'est associé à eux pour imaginer les contours de ce projet ainsi que pour préparer les débuts de son développement. Les premières réunions consacrées au projet ont permis à [Tanguy Dubois](/docs/apropos/#tanguy-dubois) de rejoindre le projet et d'apporter son expertise concernant le langage Rust. 

{{< center >}}
{{< image-legend src="/fevrier2025/setup_bubo.png" alt="Setup BuboBubo" caption="Dispositif de performance utilisé par _BuboBubo_ dans le cadre des [Instants Fertiles](https://www.athenor.com/les-rendez-vous/2024-25/instant-fertiles) au VIP de Saint-Nazaire le 14 novembre 2024." >}}
{{< /center >}}

Raphaël et Rémi avaient déjà eu l'occasion de se produire au cours de performances ou d'ateliers à l'invitation de l'Athenor, utilisant à cette fin les environnements de _live coding_ développés par Raphaël tels que [Sardine](https://sardine.raphaelforment.fr) ou [Topos](https://topos.live). Leurs performances collaboratives se doivent de composer avec un certain nombre de contraintes logicielles parfois difficiles à résoudre :
- **Synchronisation réseau** : elles nécessitent une synchronisation fine des ordinateurs et de leurs horloges musicales respectives, de pouvoir partager de l'information rapidement entre ordinateurs (séquences, événements divers, etc).
- **Précision temporelle** : elles nécessitent une gestion précise du temps de déclenchement des événements à destination des synthétiseurs, machines et autres pièces formant le dispositif de performance : problèmes de latence, etc.
- **Expressivité** : elles nécessitent de disposer de langages de programmation permettant de communiquer de manière succincte et expressive avec le matériel musical tout en définissant le code musical qui compose la performance.
- **Instrumentalité** : ces outils doivent permettre de développer un rapport immédiat et instrumental avec l'ordinateur et le code source, ici considéré comme un langage musical autant que comme un support technique et fonctionnel.

## Réunions de travail

Les premières réunion de travail autour de _Sova_, à l'IUT de Nantes, ont été centrées autour de la définition d'une architecture générale pour le logiciel. Il s'agissait avant toute chose de prendre le temps d'identifier les contraintes de conception et les objectifs visés au travers du développement de _Sova_. Une première version témoin d'une interface utilisateur avait à cette occasion été avancée par Raphaël Forment (voir Figure 2). Cette première version servait à démontrer les éléments fondamentaux qui devaient composer l'interface, et à expliquer la structure envisagée pour le séquenceur au cœur de l'application.

{{< center >}}
{{< image-legend src="/fevrier2025/bubocore.png" alt="Version prototype" caption="Prototype d'une interface utilisateur pour Sova (_TypeScript_, _Tauri_ et _Rust_)." >}}
{{< /center >}}

Pour répondre aux problèmes identifiés, le développement d'un nouveau programme à partir du langage Rust a été amorçé. Ce programme compose la partie _serveur_ de _Sova_, le cœur fonctionnel de l'application. Il règle les problèmes les plus importants que se doit de résoudre le logiciel : exécution des _scripts_, synchronisation musicale, gestion des périphériques externes, etc.


{{< center >}}
{{< image-legend src="/fevrier2025/tableau_design.jpg" alt="Tableau (session de travail)" caption="Tableau de travail autour de l'architecture interne de Sova, aujourd'hui dépassé !" >}}
{{< /center >}}

Le temps relativement court imparti pour cette première phase de réflexion n'a servi qu'à poser une première base du chantier que ce logiciel représente. Le travail sur les étapes suivantes s'est donc progressivement mis en place à distance, au cours des semaines suivantes.

## Performance au StereoLux

Cette première session de travail autour de _Sova_ s'est clôturée par une performance musicale organisée dans le cadre de la [Nuit blanche des chercheurs](https://stereolux.org/agenda/nuit-blanche-des-chercheures-2) au [StereoLux](https://stereolux.org/) de Nantes. _BuboBubo_ (Raphaël Forment) était pour l'occasion accompagné par Loïg Jezequel, chargé de _live coder_ un accompagnement visuel audioréactif à l'aide de l'environnement [Hydra](https://hydra.ojack.xyz/) (Olivia Jack).



{{< center >}}
{{< image-legend src="/fevrier2025/concert.jpg" alt="Performance au StereoLux" caption="Performance au StereoLux. À gauche : Loïg Jezequel. Au fond, projeté : code TidalCycles utilisé par _BuboBubo_. Double projection (code et visuels) autour d'un îlot central." >}}
{{< /center >}}
