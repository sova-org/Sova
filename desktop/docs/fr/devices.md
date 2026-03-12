# Périphériques

Sova est capable d'émettre des messages en MIDI et en OSC. Il est possible d'envoyer des notes, des _control changes_ et tout types de messages supportés par les protocoles cités vers des synthétiseurs, des boîtes à rythmes, des DAWs, des environnements modulaires. Sova peut dialoguer avec tout les environnements capables d'écouter et de traiter les messages émis. Le moteur audio intégré, nommé Doux, permet aussi de produire du son sans aucun équipement externe.

## Slots

Chaque périphérique occupe un slot numéroté de 1 à 16. Lorsque le code émet un événement sans préciser de périphérique, celui-ci arrive au slot 1. Le slot 1 est le slot par défaut. Les langages permettent d'envoyer des messages à n'importe quel slot. Chaque langage possèdera une syntaxe et une manière différente de le faire. Il faut donc se reporter à la documentation de chaque langage pour comprendre comment diriger un message vers l'un des périphériques.  Le slot 0 est un slot un peu particulier. Il s'agit du périphérique des `Logs`. Celui-ci est invisible dans l'interface est n'est utilisé que pour le _debug_. Les événements envoyés sur ce slot s'affichent dans les `Logs` directement.

## Connexion des périphériques

Le panneau `Périphériques` affiche le matériel disponible détecté par Sova. Les ports MIDI accessibles au sein du système apparaissent dans la liste. Il suffit de cliquer pour se connecter à ces derniers et les assigner à un slot. Il est aussi possible de créer des ports MIDI virtuels, visibles depuis un DAW ou tout autre logiciel sur la même machine. Les ports virtuels sont une fonctionnalité courante de macOS et de Linux mais ne sont pas encore pris en charge de manière standardisée sur Windows, du moins pour le moment.

Un périphérique OSC peut être créé en lui assignant un nom, une adresse IP et un numéro de port. Les événements sont alors transmis via le protocole UDP. De nombreux logiciels très répandus dans le domaine de l'informatique musicale sont conçus pour traiter les messages OSC de manière efficace : SuperCollider, Pure Data, etc. 

Le moteur audio (Doux) occupe un slot à l'instar des autres périphériques lorsque celui-ci est disponible. Une version standard de Sova inclut par défaut le moteur Doux et l'assigne au slot n°1. Ceci permet de commencer à jouer immédiatement même en l'absence de moteurs externes ou de synthétiseurs.

## Entrée MIDI

Les entrées MIDI n'occupent pas de slot. Elles alimentent le système en données : les valeurs CC reçues sont mémorisées et rendues accessibles aux scripts. Chaque langage aura une manière différente de rendre disponible les données MIDI reçues. 

## Canaux MIDI

Chaque connexion MIDI supporte les 16 canaux habituels définis par le protocole. Le canal par défaut est le premier. Un seul port suffit pour s'addresser à 16 canaux. Il est donc possible de piloter 16 instruments à partir d'un seul slot MIDI. Chaque périphérique MIDI possède également un réglage de latence (20 ms par défaut). Le thread world s'en sert pour envoyer les messages légèrement en avance sur le temps réel, compensant les potentiels délais de transmission. Il est possible d'ajuster cette valeur pour chaque périphérique.
