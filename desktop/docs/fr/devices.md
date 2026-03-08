# Périphériques

Vous voulez du son. Chaque événement produit par votre code arrive dans un slot.
Le périphérique qui occupe ce slot le transmet en MIDI, OSC ou audio. Pas de
périphérique, pas de son.

## Mise en route rapide

Ouvrez le panneau Périphériques. Trois possibilités :

1. Connecter une sortie MIDI (port matériel ou virtuel).
2. Créer un point d'accès OSC (IP + port, pour SuperCollider, Max, etc.).
3. Utiliser le moteur audio intégré (Doux) si le serveur a été lancé avec le
   support audio.

Chaque connexion est assignée à un slot (1--16). Le slot 1 est celui par
défaut -- si votre code ne précise pas de périphérique, les événements vont là.

## Sortie MIDI

Cliquez sur "Connecter MIDI" dans le panneau Périphériques. Les ports
disponibles sur votre système s'affichent. Cliquez pour connecter et assigner
à un slot.

Pour créer un port MIDI virtuel visible par d'autres applications (pratique
pour router Sova vers un DAW sur la même machine), cliquez sur "Créer un MIDI
virtuel".

En Cagire, envoyer une note vers un slot précis :

```forth
2 dev c4 note 100 vel .
```

En Bob :

```
DEV 2
>> [note: 60 vel: 100]
```

## Sortie OSC

Cliquez sur "Créer une sortie OSC" dans le panneau Périphériques. Entrez un
nom, une adresse IP cible et un port. Le point d'accès apparaît dans la liste,
prêt à être assigné à un slot.

Les événements OSC portent les mêmes paramètres que les événements MIDI.
L'application réceptrice (SuperCollider, Max, Pure Data) les interprète comme
elle l'entend.

## Slots

Sova dispose de 16 slots utilisateur (1--16) et d'un slot fixe :

- Le slot 0 est le périphérique Log. Toujours présent. Les événements envoyés
  ici s'affichent dans le panneau Journaux. Utile pour le débogage.
- Les slots 1--16 accueillent vos ports MIDI, points d'accès OSC et le moteur
  audio.

Le slot 1 est le périphérique par défaut. Les assignations persistent pour la
session, gardez-les cohérentes -- votre code fait référence aux numéros de slot
directement.

Un seul script peut adresser plusieurs slots :

```forth
1 dev "kick" snd .        ;; batterie sur le slot 1
2 dev c4 note "saw" snd . ;; synthé sur le slot 2
```

Si un slot est vide, les événements qui lui sont destinés sont ignorés
silencieusement.

## Canaux MIDI

Les canaux MIDI dans Sova vont de 1 à 16, conformément à la convention
standard. Le canal par défaut est 1. Un seul port MIDI (un slot) peut adresser
les 16 canaux :

```forth
1 chan 60 note .     ;; canal 1
10 chan 36 note .    ;; batterie sur le canal 10
```

## Entrée MIDI

Les périphériques d'entrée MIDI se connectent dans le panneau Périphériques
mais n'occupent pas de slot. Ils alimentent le système en données entrantes. En
Cagire, lire une valeur CC :

```forth
1 1 ccval    ;; CC 1 (molette de modulation), canal 1
```

Consultez l'article **MIDI** dans la documentation Cagire pour le détail
complet de l'envoi et de la réception MIDI.
