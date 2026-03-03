# Événements

Les événements sont les messages que votre code produit — notes MIDI, changements
de contrôle, messages OSC, et plus encore. Comprendre le fonctionnement des
événements vous aide à écrire du code qui fait exactement ce que vous voulez.

## Événements de notes MIDI

L'événement le plus courant est une note MIDI. Un événement de note a ces
paramètres :

- **Note** (0–127) — la hauteur. 60 = Do central.
- **Vélocité** (0–127) — l'intensité de la frappe. 0 signifie généralement
  note-off.
- **Canal** (1–16) — le canal MIDI. Par défaut : 1.
- **Durée** (battements) — combien de temps la note résonne avant qu'un
  note-off soit envoyé.
- **Périphérique** (1–16) — quel slot de périphérique reçoit l'événement.
  Par défaut : 1.

Quand un événement de note se déclenche, Sova envoie un MIDI Note On
immédiatement et planifie un Note Off correspondant après la durée spécifiée.
Vous n'avez pas besoin de gérer les note-off manuellement.

## Événements de contrôle MIDI

Au-delà des notes, MIDI offre plusieurs messages de contrôle :

- **CC (Control Change)** — contrôleurs continus (molette de modulation,
  expression, boutons personnalisés). Spécifiez un numéro de CC (0–127) et une
  valeur (0–127).
- **Program Change** — changement de patch/preset sur un synthétiseur.
  Spécifiez un numéro de programme (0–127).
- **Aftertouch** — expression sensible à la pression. Peut être par canal ou
  par note (aftertouch polyphonique).
- **Pitch Bend** — position de la molette de pitch. La plage dépend du
  synthétiseur récepteur.

Tous les événements de contrôle MIDI prennent un canal et un slot de
périphérique, comme les notes.

## Messages OSC

Les événements OSC (Open Sound Control) envoient des messages à des logiciels
externes via UDP. Un événement OSC comprend :

- **Adresse** — un pattern d'adresse OSC (par ex. `/synth/freq`).
- **Arguments** — une liste de valeurs (entiers, flottants, chaînes).
- **Périphérique** — le slot de périphérique d'un point d'accès OSC.

OSC est utile pour communiquer avec SuperCollider, Max/MSP, Pure Data, des
logiciels visuels ou toute application qui parle OSC.

## Comment les événements sont émis

La syntaxe exacte pour créer des événements diffère selon le langage, mais le
schéma général est :

1. **Définir le contexte** : choisir un slot de périphérique et un canal MIDI.
   Ceux-ci deviennent les valeurs par défaut pour les événements suivants
   jusqu'à ce qu'ils soient changés.
2. **Émettre l'événement** : utiliser la syntaxe d'événement du langage pour
   déclencher une note, un CC ou un message OSC.
3. **Attendre** : mettre en pause pendant un nombre de battements avant le
   prochain événement. Sans attente, tous les événements se déclenchent
   simultanément au début de la case.

Chaque langage a sa propre syntaxe — consultez les onglets par langage pour les
détails :

- **Bob** utilise des event maps : `>> [note: 60 vel: 100 dur: 0.5]`
- **Boinx** utilise la notation de patterns pour les séquences rythmiques.
- **Cagire** utilise des mots basés sur la pile pour empiler et émettre des
  événements.
- **BaLi** utilise la construction d'événements par expressions.

## Routage canal et périphérique

Chaque événement porte une valeur de canal et de périphérique. Vous les définissez
avant l'émission :

- **Périphérique** sélectionne le slot de sortie (1–16). Le slot 0 est la
  console de journaux.
- **Canal** sélectionne le canal MIDI (1–16). Ignoré pour les événements OSC.

Vous pouvez changer de périphérique et de canal en cours de script pour router
différents événements vers différentes sorties au sein d'une même case. Par
exemple, vous pourriez envoyer les notes de mélodie vers le périphérique 1 /
canal 1 et les notes de basse vers le périphérique 2 / canal 3.

## Timing des événements

Les événements sont envoyés avec un timing précis par le thread monde :

- Les événements MIDI sont envoyés avec 2 ms d'anticipation pour une
  synchronisation serrée.
- Les événements OSC sont envoyés avec 20 ms d'anticipation.

Le planificateur prépare les événements ~30 ms en avance sur le temps réel. Cela
signifie que les événements sont mis en file d'attente et envoyés au moment exact,
pas déclenchés au hasard.

## Astuces

- Utilisez le périphérique **Log** (slot 0) pour inspecter les événements
  produits par votre code avant de les router vers une vraie sortie.
- Une note avec une vélocité de 0 est traitée comme un note-off par la plupart
  des synthés.
- OSC vous permet de contrôler n'importe quoi — lumières, visuels, autres
  logiciels — pas seulement du son.
- Gardez la durée à l'esprit : des notes qui se chevauchent (longues durées +
  attentes courtes) produisent des accords. Des notes qui ne se chevauchent pas
  (durée ≤ attente) produisent du staccato.
