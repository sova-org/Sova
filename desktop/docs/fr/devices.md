# Peripheriques

Vous voulez du son. Chaque evenement produit par votre code arrive dans un slot.
Le peripherique qui occupe ce slot le transmet en MIDI, OSC ou audio. Pas de
peripherique, pas de son.

## Mise en route rapide

Ouvrez le panneau Peripheriques. Trois possibilites :

1. Connecter une sortie MIDI (port materiel ou virtuel).
2. Creer un point d'acces OSC (IP + port, pour SuperCollider, Max, etc.).
3. Utiliser le moteur audio integre (Doux) si le serveur a ete lance avec le
   support audio.

Chaque connexion est assignee a un slot (1--16). Le slot 1 est celui par
defaut -- si votre code ne precise pas de peripherique, les evenements vont la.

## Sortie MIDI

Cliquez sur "Connecter MIDI" dans le panneau Peripheriques. Les ports
disponibles sur votre systeme s'affichent. Cliquez pour connecter et assigner
a un slot.

Pour creer un port MIDI virtuel visible par d'autres applications (pratique
pour router Sova vers un DAW sur la meme machine), cliquez sur "Creer un MIDI
virtuel".

En Cagire, envoyer une note vers un slot precis :

```forth
2 dev c4 note 100 vel .
```

En Bob :

```
DEV 2
>> [note: 60 vel: 100]
```

## Sortie OSC

Cliquez sur "Creer une sortie OSC" dans le panneau Peripheriques. Entrez un
nom, une adresse IP cible et un port. Le point d'acces apparait dans la liste,
pret a etre assigne a un slot.

Les evenements OSC portent les memes parametres que les evenements MIDI.
L'application receptrice (SuperCollider, Max, Pure Data) les interprete comme
elle l'entend.

## Slots

Sova dispose de 16 slots utilisateur (1--16) et d'un slot fixe :

- Le slot 0 est le peripherique Log. Toujours present. Les evenements envoyes
  ici s'affichent dans le panneau Journaux. Utile pour le debogage.
- Les slots 1--16 accueillent vos ports MIDI, points d'acces OSC et le moteur
  audio.

Le slot 1 est le peripherique par defaut. Les assignations persistent pour la
session, gardez-les coherentes -- votre code fait reference aux numeros de slot
directement.

Un seul script peut adresser plusieurs slots :

```forth
1 dev "kick" snd .        ;; batterie sur le slot 1
2 dev c4 note "saw" snd . ;; synthe sur le slot 2
```

Si un slot est vide, les evenements qui lui sont destines sont ignores
silencieusement.

## Canaux MIDI

Les canaux MIDI dans Sova vont de 1 a 16, conformement a la convention
standard. Le canal par defaut est 1. Un seul port MIDI (un slot) peut adresser
les 16 canaux :

```forth
1 chan 60 note .     ;; canal 1
10 chan 36 note .    ;; batterie sur le canal 10
```

## Entree MIDI

Les peripheriques d'entree MIDI se connectent dans le panneau Peripheriques
mais n'occupent pas de slot. Ils alimentent le systeme en donnees entrantes. En
Cagire, lire une valeur CC :

```forth
1 1 ccval    ;; CC 1 (molette de modulation), canal 1
```

Consultez l'article **MIDI** dans la documentation Cagire pour le detail
complet de l'envoi et de la reception MIDI.
