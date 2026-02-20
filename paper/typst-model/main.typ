#import "jim.typ": jim

#show: jim.with(
  title: "Sova",
  authors: (
    (name: "Raphaël Forment", affiliation: "Indépendant"),
    (name: "Tanguy Dubois", affiliation: "Laboratoire LS2N"),
    (name: "Loïg Jezequel", affiliation: "Laboratoire LS2N"),
  ),
  abstract: [
    Blabla Sova Blabla Sova.
  ],
)

= Introduction <sec:introduction>

Ce modèle précise les règles pour une bonne présentation des communications à proposer aux JIM 2026. Il s'inspire du modèle proposé les années précédentes. Merci de le suivre afin de permettre une présentation unifiée des actes.

= Taille de la page <sec:page_size>

Les actes seront imprimés au format A4 (21 x 29.7 cm). Le contenu de chaque page doit pouvoir tenir dans un rectangle de (17 x 24.7 cm) centré sur la page, commençant à 2 cm du haut de la page et s'arrêtant à 3 cm du bas de la page. Les marges gauche et droite doivent être de 2 cm. Le texte est présenté sur deux colonnes (8,1cm) avec une gouttière de 0,8 cm. Le texte doit être justifié à gauche et à droite.

= Police de caractères <sec:typeset_text>

== Corps du texte <subsec:body>

Utiliser la police Times 10 pt (points). N'utiliser une police sans serif ou non proportionnelle que pour des raisons particulières, par exemple pour distinguer des lignes de code du reste du texte.

== Titre et auteurs

Le titre est en Times 14 pt, gras, majuscule, centré. Les noms des auteurs sont centrés.
_Pour la soumission, en *double-aveugle*, ne pas indiquer les auteurs ni les organisations, mais laisser tel quel le bloc._
Si l'adresse est la même pour tous les auteurs, elle ne doit figurer qu'une seule fois, centrée. Dans le cas contraire, elle doit apparaître sous le nom de chaque auteur.

== Numéro de page, haut de page et bas de page

Ne pas inclure de numéro de page, de haut de page ou de bas de page lors de votre soumission. Ils seront ajoutés par l'éditeur.

= Sections

Les titres de sections sont en Times, 10 pt gras, centrés avec 1 ligne d'espace au-dessus du titre de section, et 1/2 espace au-dessous. Pour un titre de section immédiatement suivi d'un titre de sous-section, ne pas additionner les deux espaces.

== Sous-sections

Les titres de sous-sections sont en Times 10 pt alignés à gauche, avec une ligne d'espace au-dessus, et 1/2 ligne d'espace au-dessous.

=== Sous-sous-sections

Les sous-sous-sections sont en Times 10 pt italique, alignés à gauche, avec 1 ligne d'espace au-dessus et 1/2 ligne d'espace au-dessous.

On évitera d'utiliser plus de trois niveaux de section.

= Notes de bas de page et Figures

== Notes de bas de page

Indiquer la note de bas de page avec un numéro dans le texte#footnote[Ceci est une note de bas de page]. Utiliser la police Times 8 pt. Placer les notes en bas de chaque page où elles vont apparaître. Faire précéder la note d'une ligne horizontale de 0,5 pt.

== Illustrations, figures et tableaux

Toutes les illustrations devront être centrées dans une colonne, propres et lisibles (Figure 1). L'impression des actes sera en noir et blanc. Les figures doivent donc faire sens en noir et blanc. Les numéros de figure, de tableau et leur légende doivent toujours apparaître en dessous de la figure. Laisser une ligne d'espace entre la figure et sa légende. Chaque figure ou tableau est numéroté consécutivement. Les légendes seront présentées en Times 10 pt et indentées. Placer les illustrations aussi près des références que possible. Elles peuvent être placées au centre de la page, traversant les deux colonnes, dans une limite de 17cm.

#figure(
  table(
    columns: 2,
    stroke: 0.5pt,
    [Texte], [Valeur],
    [hello jim], [1073],
  ),
  caption: [La légende du tableau devra être placée sous le tableau.],
) <tab:example>

#figure(
  box(stroke: 0.5pt, image("figure.pdf", width: 100%)),
  caption: [La légende de la figure devra être placée sous la figure.],
) <fig:example>

= Equations

Les équations devront être placées sur des lignes séparées et numérotées. Le numéro devra être placé à droite.

$ E = m c^2 $

= Citations

Toutes les références bibliographiques des citations devront être listées dans la section "References", numérotées et en ordre alphabétique. Toutes les références listées devront être citées dans le texte. Quand vous vous référez au document dans le texte, précisez son numéro [1].

= #smallcaps[References]

#set par(first-line-indent: 0pt, hanging-indent: 1.5em)

[1] Author, E. "Titre du papier", _Proceedings of the International Symposium on Music Information Retrieval_, Plymouth, USA, 2000.

[2] Untel, A. _Titre du livre_. L'Armada, Paris, 2005.
