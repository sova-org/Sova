#import "jim.typ": jim

#show: jim.with(
  title: "Sova (Сова) : un environnement de programmation polyglotte, une machine virtuelle, un serveur et un moteur audio pour le live coding collaboratif",
  authors: (
    (name: "Raphaël Forment", affiliation: "Indépendant (ECLLA LS2N)"),
    (name: "Tanguy Dubois", affiliation: "Laboratoire LS2N"),
    (name: "Loïg Jezequel", affiliation: "Laboratoire LS2N"),
  ),
  abstract: [
    Sova est un environnement de programmation libre et _open source_ (licence AGPL 3.0#footnote[Lien vers le texte de la licence hébergé par le projet GNU : https://www.gnu.org/licenses/agpl-3.0.en.html. Consulté le 20 février 2026.]) pensé pour la pratique du _live coding_ musical et implémenté sur des technologies nouvelles : langage _Rust_, protocole _Ableton Link_, etc. Sova se compose d'une machine virtuelle dédiée à la création de langages musicaux événementiels, d'une interface client / serveur, d'un moteur dédié à la synthèse sonore et à l'échantillonnage et de plusieurs interfaces utilisateur. Sova est le résultat temporaire d'une collaboration art-sciences établie par l'Athénor CNCM#footnote[Lien vers le site du centre de création : https://athenor.com. Consulté le 20 février 2026.] entre Raphaël Forment et le laboratoire LS2N#footnote[Lien vers le site du laboratoire : https://www.ls2n.fr/. Consulté le 20 février 2026.] de l'Université de Nantes au cours de l'année 2025. Le projet Sova est aujourd'hui au cœur d'une initiative de médiation art-sciences portée par l'Athénor CNCM au sein de plusieurs établissements scolaires de la région Pays de la Loire. 
  ],
)

#figure(
  image("sova_screenshot.png", width: 100%),
  caption: [Capture d'écran de l'interface utilisateur multijoueur de Sova : `sova-frontend` (février 2026).]
) <fig:sova_screenshot>

= Introduction <sec:introduction>

La pratique du _live coding_ en informatique musicale et dans les arts audiovisuels connaît un essor notable depuis plusieurs décennies. Cet essor se manifeste par la constitution d'un réseau de recherche pérenne#footnote[Voir à titre d'exemple la liste des éditions et les actes publiés des _International Conference on Live Coding_ : https://iclc.toplap.org/. Consulté le 20 février 2026.] @blackwell2022live , l'ancrage d'un mouvement culturel et de réseaux artistiques internationaux et la publication fréquente de nouveaux outils facilitant l'accès à cette pratique.

#figure(
  image("scene.png", width: 100%),
  caption: [Schéma de la structure d'une scène Sova.],
) <fig:scene>


Ce modèle précise les règles pour une bonne présentation des communications à proposer aux JIM 2026. Il s'inspire du modèle proposé les années précédentes. Merci de le suivre afin de permettre une présentation unifiée des actes.

= État de l'art <sec:page_size>

== Un champ dominé par quelques logiciels

= Architecture <sec:architecture>

== `core` : 

== `server`: interfaces réseau

== `langs` : langages, compilateurs, interpréteurs

= Machine virtuelle <sec:machine_virtuelle>

= Moteur audio <sec:moteur_audio>

Doux est un moteur...

= Interfaces <sec:machine_virtuelle>

Plusieurs interfaces utilisateurs ont pu être développées pour le bien du projet. 

== Interfaces en ligne de commande

== Interfaces graphiques

=== TUI : Terminal User Interface

=== GUI : Graphical User Interface

= Langages <sec:langages>

Plusieurs langages de programmation orientés pour l'improvisation musicale sont en cours de développement pour Sova. Chacun d'entre eux se destine à étudier l'une des possibilités ouvertes par l'architecture logicielle et par la machine virtuelle sus-décrite.

== Langages compilés : `Bob`, `Bali`
 
== Langages interprétés : `Boinx`, `Cagire`



= Limites et ...

= Conclusion et travaux futurs

L'ambition de cet article réside seulement dans le fait de présenter et de témoigner de l'avancée du projet _Sova_, sans postuler ni sur ses objectifs finaux ni sur les futures initiatives construites autour du projet. _Sova_ est aussi bien conçu pour permettre le développement de nouveaux langages dédiés au _live coding_ que pour offrir une base solide pour la pédagogie et l'enseignement des techniques et des pratiques du _live coding_. De nouvelles étapes de travail sont toutefois d'ores et déjà établies pour permettre la pérennité du travail de développement : expériences de création, nouvelles expériences de médiation, etc.

== Expériences pédagogiques et médiation

Sova est actuellement employé dans le cadre d'un projet de médiation en milieu scolaire porté par l'Athénor CNCM et le laboratoire LS2N au sein de la Région Pays de la Loire. Au cours de l'hiver et du printemps 2026, plusieurs établissements scolaires de la ville de Nantes, de Saint-Nazaire et Guérande ont pu commencer à expérimenter l'utilisation collective de _Sova_ pour la mise en place d'une création commune prévue pour mai 2026.

== Productions artistiques

_Sova_ sera employé au cours de la saison 2026 pour la création d'une performance musicale et d'une installation interactive dans le cadre du projet _Useful Fictions \#6_#footnote[Lien vers l'appel à candidature pour le projet : https://www.reseau-tras.eu/appel-a-candidatures-leau-comme-horizon-useful-fictions-6-jusquau-8-mars/] (Laboratoire n°3), en collaboration avec Olivier Doaré. Le logiciel est aussi dévoilé progressivement à la communauté existante du _live coding_ au travers de plusieurs autres initiatives : _algoraves_, conférences#footnote[Voir par exemple l'initiative portée par le collectif _TOPLAP Italia_ pour l'organisation d'une conférence en mars 2026 : https://equinoxtoplap.it/. Consulté le 20 février 2026.]

 La publication en _open source_ des premières versions du logiciel permettra aussi 


#bibliography("references.bib", title: smallcaps[References], style: "ieee")
