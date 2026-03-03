# Variables

Les variables vous permettent de stocker et de partager des données entre
scripts. Le système de variables de Sova est organisé par **portée** — l'endroit
où la variable réside détermine qui peut la voir et combien de temps elle
persiste.

## Portées

| Portée | Durée de vie | Visibilité | Cas d'usage |
|--------|-------------|------------|-------------|
| Globale | Session entière | Tous les scripts de la scène | État partagé, paramètres globaux |
| Ligne | Durée de vie de la ligne | Toutes les cases de cette ligne | État par piste, compteurs |
| Case | Durée de vie de la case | Le script de cette case | État par cellule, données d'itération |
| Instance | Exécution unique | Une exécution du script | Registres temporaires, travail local |

### Variables globales

Les variables globales sont partagées dans toute la scène. N'importe quel script
dans n'importe quelle ligne et case peut les lire et les écrire. Elles
persistent tant que la session est en cours.

Utilisez les globales pour les valeurs sur lesquelles plusieurs lignes doivent
s'accorder : une note fondamentale, une gamme, un seuil de probabilité, une
transposition globale.

### Variables de ligne

Les variables de ligne appartiennent à une ligne spécifique. Toutes les cases
de cette ligne peuvent y accéder, mais les scripts des autres lignes ne le
peuvent pas. Elles persistent lors des changements de case au sein de la ligne.

Utilisez les variables de ligne pour l'état par piste : un compteur de pas qui
avance à chaque boucle de la ligne, ou un tableau de mélodie que les cases
lisent.

### Variables de case

Les variables de case appartiennent à une case spécifique. Elles persistent
entre les répétitions de cette case mais se réinitialisent quand la ligne passe
à la case suivante.

Utilisez les variables de case pour un état qui doit survivre aux répétitions
mais ne doit pas se propager aux autres cases.

### Variables d'instance

Les variables d'instance n'existent que pendant une seule exécution d'un script.
Elles sont créées à neuf chaque fois que la case est jouée et supprimées
ensuite. C'est la portée la plus locale — essentiellement des registres
temporaires.

Dans les langages compilés, les variables d'instance comme `Instance("0")` et
`Instance("1")` servent de registres de travail pour la VM.

## Comment les portées se rapportent à la scène

La hiérarchie des portées reflète la hiérarchie de la scène :

```
Scène ──── Variables globales
 └─ Ligne ──── Variables de ligne
     └─ Case ──── Variables de case
         └─ Exécution ──── Variables d'instance
```

Les données circulent naturellement : une variable globale définie dans une
ligne est immédiatement visible dans une autre. Une variable de ligne définie
dans la case 1 est visible dans la case 2 quand la ligne avance. Les variables
d'instance sont isolées à une seule exécution et disparaissent après.

## Valeurs intégrées en lecture seule

Chaque langage expose certaines valeurs intégrées que vous pouvez lire mais pas
écrire. Elles proviennent de la portée **Environnement** et fournissent le
contexte de l'exécution en cours :

- Position actuelle en battements
- Tempo actuel
- Génération de nombres aléatoires
- Index de case, index de ligne
- Compteur d'itération (combien de fois la case actuelle s'est répétée)

Les noms exacts et la syntaxe d'accès varient selon le langage — consultez la
référence de chaque langage pour la liste complète.

## Astuces

- Minimisez les globales. Si seule une ligne a besoin d'une valeur, utilisez
  une variable de ligne à la place.
- Utilisez les variables de case pour les accumulateurs qui se réinitialisent
  naturellement quand la ligne passe à la section suivante.
- Le système de variables est le principal moyen de communication entre scripts.
  Deux cases dans des lignes différentes peuvent se coordonner en lisant et
  écrivant la même variable globale.
