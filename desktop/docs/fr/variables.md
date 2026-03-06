# Variables

Les variables vous permettent de stocker et de partager des données entre
scripts. Le système de variables de Sova est organisé par **portée** — l'endroit
où la variable réside détermine qui peut la voir et combien de temps elle
persiste.

## Portées

- **Globale** — Session entière. Visible par tous les scripts de la scène. Pour l'état partagé, paramètres globaux.
- **Ligne** — Durée de vie de la ligne. Visible par toutes les frames de cette ligne. Pour l'état par piste, compteurs.
- **Frame** — Durée de vie de la frame. Visible par le script de cette frame. Pour l'état par cellule, données d'itération.
- **Instance** — Exécution unique. Visible par une exécution du script. Pour les registres temporaires, travail local.

### Variables globales

Les variables globales sont partagées dans toute la scène. N'importe quel script
dans n'importe quelle ligne et frame peut les lire et les écrire. Elles
persistent tant que la session est en cours.

Utilisez les globales pour les valeurs sur lesquelles plusieurs lignes doivent
s'accorder : une note fondamentale, une gamme, un seuil de probabilité, une
transposition globale.

### Variables de ligne

Les variables de ligne appartiennent à une ligne spécifique. Toutes les frames
de cette ligne peuvent y accéder, mais les scripts des autres lignes ne le
peuvent pas. Elles persistent lors des changements de frame au sein de la ligne.

Utilisez les variables de ligne pour l'état par piste : un compteur de pas qui
avance à chaque boucle de la ligne, ou un tableau de mélodie que les frames
lisent.

### Variables de frame

Les variables de frame appartiennent à une frame spécifique. Elles persistent
entre les répétitions de cette frame mais se réinitialisent quand la ligne passe
à la frame suivante.

Utilisez les variables de frame pour un état qui doit survivre aux répétitions
mais ne doit pas se propager aux autres frames.

### Variables d'instance

Les variables d'instance n'existent que pendant une seule exécution d'un script.
Elles sont créées à neuf chaque fois que la frame est jouée et supprimées
ensuite. C'est la portée la plus locale — essentiellement des registres
temporaires.

Dans les langages compilés, les variables d'instance comme `Instance("0")` et
`Instance("1")` servent de registres de travail pour la VM.

## Comment les portées se rapportent à la scène

La hiérarchie des portées reflète la hiérarchie de la scène :

```
Scène ──── Variables globales
 └─ Ligne ──── Variables de ligne
     └─ Frame ──── Variables de frame
         └─ Exécution ──── Variables d'instance
```

Les données circulent naturellement : une variable globale définie dans une
ligne est immédiatement visible dans une autre. Une variable de ligne définie
dans la frame 1 est visible dans la frame 2 quand la ligne avance. Les variables
d'instance sont isolées à une seule exécution et disparaissent après.

## Valeurs intégrées en lecture seule

Chaque langage expose certaines valeurs intégrées que vous pouvez lire mais pas
écrire. Elles proviennent de la portée **Environnement** et fournissent le
contexte de l'exécution en cours :

- Position actuelle en beats
- Tempo actuel
- Génération de nombres aléatoires
- Index de frame, index de ligne
- Compteur d'itération (combien de fois la frame actuelle s'est répétée)

Les noms exacts et la syntaxe d'accès varient selon le langage — consultez la
référence de chaque langage pour la liste complète.

## Astuces

- Minimisez les globales. Si seule une ligne a besoin d'une valeur, utilisez
  une variable de ligne à la place.
- Utilisez les variables de frame pour les accumulateurs qui se réinitialisent
  naturellement quand la ligne passe à la section suivante.
- Le système de variables est le principal moyen de communication entre scripts.
  Deux frames dans des lignes différentes peuvent se coordonner en lisant et
  écrivant la même variable globale.
