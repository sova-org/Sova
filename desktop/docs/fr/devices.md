# Périphériques

Les périphériques sont les sorties qui transmettent votre musique au monde
extérieur. Chaque événement produit par votre code est routé vers un
périphérique — un port MIDI, un point d'accès OSC, le moteur audio intégré ou
la console de journaux.

## La carte des périphériques

Sova utilise une **carte de périphériques** avec 16 slots numérotés :

- **Slot 0** — le périphérique Log. Toujours présent, non assignable par
  l'utilisateur. Les événements envoyés ici apparaissent dans le panneau
  Journaux. Utile pour le débogage.
- **Slots 1–16** — assignables par l'utilisateur. Vous y placez vos ports MIDI,
  points d'accès OSC et connexions au moteur audio.

Quand votre code émet un événement, il cible un **numéro de slot**. Le
périphérique qui occupe ce slot reçoit l'événement. Si le slot est vide,
l'événement est silencieusement ignoré.

Le **slot par défaut est 1** — si votre code ne spécifie pas de périphérique,
les événements vont vers le slot 1.

## Types de périphériques

- **Sortie MIDI** — Un port MIDI matériel ou logiciel de votre système
- **Sortie MIDI virtuelle** — Un port MIDI virtuel créé par Sova (visible dans les autres applications)
- **Sortie OSC** — Un point d'accès UDP (adresse IP + port) pour Open Sound Control
- **Moteur audio** — Le synthétiseur intégré Doux (voir l'article Moteur audio)
- **Log** — La console de débogage (slot 0, toujours présent)

Les périphériques d'entrée MIDI peuvent aussi être connectés pour recevoir du
MIDI externe, mais ils n'occupent pas de slots — ils alimentent le système
différemment.

## Le panneau Périphériques

Ouvrez le panneau Périphériques pour gérer vos connexions :

- **Connecter MIDI** : liste les ports MIDI disponibles sur votre système.
  Cliquez pour connecter.
- **Créer un MIDI virtuel** : crée un nouveau port MIDI virtuel que les autres
  applications peuvent voir et recevoir.
- **Créer une sortie OSC** : spécifiez un nom, une adresse IP cible et un
  numéro de port.
- **Assigner à un slot** : assignez les périphériques connectés aux slots 1–16.
- **Désassigner** : retirez un périphérique de son slot sans le déconnecter.

## Routage des événements depuis le code

Dans vos scripts, vous contrôlez quel périphérique reçoit les événements en
définissant la variable de périphérique. La syntaxe exacte dépend du langage —
consultez la référence de chaque langage pour les détails. L'idée générale :

- Définissez le périphérique sur un numéro de slot (1–16) avant d'émettre des
  événements.
- Les événements héritent du réglage de périphérique courant.
- Vous pouvez changer de périphérique en cours de script pour router différents
  événements vers différentes sorties.

Par exemple, le slot 1 pourrait être votre synthé, le slot 2 votre boîte à
rythmes, et le slot 3 une connexion OSC vers un programme visuel. Un seul
script peut adresser les trois.

## Canaux MIDI

Les canaux MIDI sont **indexés à partir de 1** dans l'interface et le code de
Sova (1–16), conformément à la convention MIDI standard. Le canal par défaut
est 1.

Chaque événement peut cibler un canal spécifique indépendamment du slot de
périphérique. Cela signifie qu'un port MIDI (un slot de périphérique) peut
adresser les 16 canaux MIDI.

## Astuces

- Gardez les assignations de slots cohérentes entre les sessions — votre code
  fait référence aux numéros de slot, donc réorganiser les périphériques entre
  les slots cassera le routage.
- Utilisez le périphérique Log (slot 0) pendant le développement pour voir
  exactement quels événements votre code produit avant de les envoyer vers une
  vraie sortie.
- Les sorties MIDI virtuelles sont le moyen le plus simple de router Sova vers
  un DAW sur la même machine.
