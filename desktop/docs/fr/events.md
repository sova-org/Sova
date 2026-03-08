# Evenements

Votre code produit des evenements : messages MIDI, messages OSC, ou commandes
audio envoyees aux peripheriques.

## Notes MIDI

Un evenement de note envoie un Note On, puis un Note Off quand la duree est
ecoulee. Vous n'envoyez jamais de note-off vous-meme.

Bob :

```
>> [note: 60 vel: 100 dur: 0.5]
```

Cagire :

```forth
60 note 100 vel 0.5 dur .
```

Parametres : hauteur (0-127), velocite (0-127), duree (beats), canal (1-16),
peripherique (1-16). Par defaut : velocite 100, duree 0.5, canal 1,
peripherique 1.

## Control Change

Les CC controlent les potentiometres, faders et parametres sur les
synthetiseurs externes.

Bob :

```
>> [cc: 74 val: 100]
```

Cagire :

```forth
74 ccnum 100 ccout .
```

## Pitch bend

Plage : -1.0 (fond bas) a 1.0 (fond haut), centre 0.0.

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

OSC envoie des messages par UDP vers SuperCollider, Max/MSP, Pure Data, ou
tout ce qui parle OSC.

Bob :

```
>> [addr: "/synth" freq: 440 amp: 0.5]
```

`addr` definit l'adresse OSC. Chaque autre cle devient un argument. Routez
vers un slot de peripherique OSC avec `dev`.

## Routage peripherique et canal

Chaque evenement porte un slot de peripherique et un canal MIDI.

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

Le peripherique selectionne le slot de sortie (1-16). Le canal selectionne
le canal MIDI. Le slot 0 est la console de log -- utilisez-le pour inspecter
les evenements avant de les router vers une vraie sortie. Vous pouvez changer
de peripherique et de canal en cours de script.

## Accords et sequences

Sans attente, les evenements se declenchent simultanement -- des accords :

```
>> [note: 60] >> [note: 64] >> [note: 67]
```

Ajoutez des attentes pour une sequence :

```
>> [note: 60] WAIT 0.5 >> [note: 64] WAIT 0.5 >> [note: 67]
```

En Cagire, `at` avec `arp` place une note par creneau :

```forth
0 0.33 0.66 at
c4 e4 g4 arp note sine snd .
```

Voir l'article **Timing** pour les details sur `at` et `arp`.

## Lire l'entree MIDI

Cagire lit les valeurs CC entrantes depuis des controleurs materiels :

```forth
74 1 ccval 127 / 200 2740 range lpf
```

Lit le CC 74 sur le canal 1, normalise en 0.0-1.0, met a l'echelle sur
200-2740, applique comme frequence de coupure. Consultez la reference de
chaque langage pour l'API d'entree complete.
