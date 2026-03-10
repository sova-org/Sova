# Événements

Votre code produit des événements : messages MIDI, messages OSC ou commandes
audio envoyées aux périphériques.

## Notes MIDI

Un événement de note envoie un Note On, puis un Note Off quand la durée est
écoulée. On n'envoie jamais de Note Off soi-même.

Bob :

```
>> [note: 60 vel: 100 dur: 0.5]
```

Cagire :

```forth
60 note 100 vel 0.5 dur .
```

Paramètres : hauteur (0–127), vélocité (0–127), durée (beats), canal (1–16),
périphérique (1–16). Par défaut : vélocité 100, durée 0.5, canal 1,
périphérique 1.

## Control Change

Les CC contrôlent les potentiomètres, faders et paramètres sur les
synthétiseurs externes.

Bob :

```
>> [cc: 74 val: 100]
```

Cagire :

```forth
74 ccnum 100 ccout .
```

## Pitch bend

Plage : -1.0 (fond bas) à 1.0 (fond haut), centre 0.0.

```forth
0.5 bend .
```

## Program Change

```
>> [pc: 12]
```

```forth
12 program .
```

## Messages OSC

OSC envoie des messages par UDP vers SuperCollider, Max/MSP, Pure Data ou
toute application compatible.

Bob :

```
>> [addr: "/synth" freq: 440 amp: 0.5]
```

`addr` définit l'adresse OSC. Chaque autre clé devient un argument. Routez
vers un slot de périphérique OSC avec `dev`.

## Routage périphérique et canal

Chaque événement porte un slot de périphérique et un canal MIDI.

Bob :

```
DEV 1
>> [note: 60 chan: 0]
DEV 2
>> [note: 48 chan: 2]
```

Cagire :

```forth
1 dev 60 note .
2 dev 48 note 3 chan .
```

Le périphérique sélectionne le slot de sortie (1–16). Le canal sélectionne le
canal MIDI. Le slot 0 est la console de log — on l'utilise pour inspecter les
événements avant de les router vers une sortie réelle. On peut changer de
périphérique et de canal en cours de script.

## Accords et séquences

Sans attente, les événements se déclenchent simultanément — des accords :

```
>> [note: 60] >> [note: 64] >> [note: 67]
```

Ajoutez des attentes pour obtenir une séquence :

```
>> [note: 60] WAIT 0.5 >> [note: 64] WAIT 0.5 >> [note: 67]
```

En Cagire, `at` avec `arp` place une note par créneau :

```forth
0 0.33 0.66 at
c4 e4 g4 arp note sine snd .
```

Voir l'article **Timing** pour les détails sur `at` et `arp`.

## Lire l'entrée MIDI

Cagire lit les valeurs CC entrantes depuis des contrôleurs matériels :

```forth
74 1 ccval 127 / 200 2740 range lpf
```

Lit le CC 74 sur le canal 1, normalise en 0.0–1.0, met à l'échelle sur
200–2740 et applique le résultat comme fréquence de coupure. Consultez la
référence de chaque langage pour l'API d'entrée complète.
