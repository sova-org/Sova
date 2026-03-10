# Moteur audio

Doux est le synthétiseur intégré de Sova. Il tourne dans le serveur et produit
l'audio directement, sans logiciel ni matériel externe. Doux permet de commencer
à produire du son immédiatement — un point d'entrée accessible pour les
débutants comme un outil complet pour le sound design avancé. Si le serveur a
été lancé avec le support audio (comportement par défaut), Doux est disponible
dès le démarrage.

## Capacités de Doux

Oscillateurs (sinus, dent de scie, carré, triangle, bruit), lecture
d'échantillons, filtres (passe-bas, passe-haut, passe-bande, variantes
ladder), reverb, delay, distorsion, chorus, phaser, synthèse FM, compression
et enregistrement en direct vers des échantillons réutilisables. La liste
complète des paramètres se trouve dans la référence de langage Cagire.

## Panneau audio

Ouvrez le panneau Audio pour configurer le moteur :

- Périphérique de sortie — quelle interface audio utiliser.
- Dossiers d'échantillons — répertoires où Doux charge ses samples.
- Voix — nombre de voix de synthèse simultanées.

Le panneau indique si le moteur tourne.

## Oscilloscope, spectre, VU-mètre

Trois panneaux de visualisation surveillent la sortie audio :

- L'oscilloscope affiche la forme d'onde. Détachable dans une fenêtre séparée.
- Le spectre affiche le contenu fréquentiel. Également détachable.
- Le VU-mètre affiche le niveau du signal.

Ils se mettent à jour en temps réel depuis le serveur. Utiles pour le sound
design et comme élément visuel en performance.

## Utiliser Doux depuis Cagire

Cagire est le langage principal pour piloter Doux. Un échantillon :

```forth
"kick" snd .
```

Un son en dent de scie filtré avec reverb :

```forth
"saw" snd c4 note 0.5 gain 800 lpf 0.3 verb .
```

Synthèse FM avec enveloppe :

```forth
"sine" snd c4 note 200 fm 2 fmh 0.01 att 0.3 dec .
```

Enregistrement en direct, puis lecture avec effets :

```forth
"loop" rec              ;; démarrer l'enregistrement
```

```forth
"loop" rec              ;; arrêter, l'échantillon est enregistré
loop snd 0.5 speed 800 lpf 0.4 verb .
```

Compression sidechain entre orbites :

```forth
0 orbit "kick" snd .                 ;; kick sur l'orbite 0
1 orbit "saw" snd c3 note 0.8 comp 0 corbit .  ;; ducker le synthé depuis l'orbite 0
```

## Utiliser Doux depuis d'autres langages

Bob, Boinx et BaLi peuvent envoyer des événements de notes au slot de Doux.
Doux répond aux messages MIDI Note On/Off avec sa voix par défaut :

```
DEV 2
>> [note: 60 vel: 100]
WAIT 1
>> [note: 64 vel: 80]
```

Pour un contrôle complet de la synthèse (filtres, effets, FM), utilisez Cagire.

## Mise en route

Doux est activé par défaut. Lorsque vous démarrez le serveur intégré depuis
l'application, le moteur audio démarre automatiquement et occupe un slot de
périphérique (vérifiez le panneau Périphériques pour identifier lequel).

1. Ouvrez le panneau Audio et sélectionnez votre périphérique de sortie.
2. Vérifiez que le moteur tourne.
3. Routez vos événements vers le slot de Doux.

En session multijoueur, tous les musiciens partagent le même moteur — chaque
client peut déclencher du son. Si vous n'utilisez que du matériel MIDI externe,
ignorez Doux. Il ne consomme rien lorsqu'il est inactif.
