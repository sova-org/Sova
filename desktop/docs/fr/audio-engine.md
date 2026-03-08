# Moteur audio

Doux est le synthetiseur integre de Sova. Il tourne dans le serveur, produit
l'audio directement, sans logiciel ni materiel externe. Si le serveur a ete
lance avec le support audio (c'est le cas par defaut), Doux est disponible des
le demarrage.

## Ce que Doux sait faire

Oscillateurs (sinus, dent de scie, carre, triangle, bruit), lecture
d'echantillons, filtres (passe-bas, passe-haut, passe-bande, variantes
ladder), reverb, delay, distorsion, chorus, phaser, synthese FM, compression,
et enregistrement en direct vers des echantillons reutilisables. La liste
complete des parametres se trouve dans la **Language Reference** de Cagire.

## Panneau Audio

Ouvrez le panneau Audio pour configurer le moteur :

- Peripherique de sortie -- quelle interface audio utiliser.
- Dossiers d'echantillons -- repertoires ou Doux charge ses samples.
- Voix -- nombre de voix de synthese simultanees.

Le panneau indique si le moteur tourne.

## Oscilloscope, Spectre, VU-metre

Trois panneaux de visualisation surveillent la sortie audio :

- L'oscilloscope affiche la forme d'onde. Detachable dans une fenetre separee.
- Le spectre affiche le contenu frequentiel. Aussi detachable.
- Le VU-metre affiche le niveau du signal.

Ils se mettent a jour en temps reel depuis le serveur. Utiles pour le sound
design et comme element visuel en performance.

## Utiliser Doux depuis Cagire

Cagire est le langage principal pour piloter Doux. Un sample :

```forth
"kick" snd .
```

Un son en dent de scie filtre avec reverb :

```forth
"saw" snd c4 note 0.5 gain 800 lpf 0.3 verb .
```

Synthese FM avec enveloppe :

```forth
"sine" snd c4 note 200 fm 2 fmh 0.01 att 0.3 dec .
```

Enregistrement en direct, puis lecture avec effets :

```forth
"loop" rec              ;; demarrer l'enregistrement
```

```forth
"loop" rec              ;; arreter, le sample est enregistre
loop snd 0.5 speed 800 lpf 0.4 verb .
```

Compression sidechain entre orbites :

```forth
0 orbit "kick" snd .                 ;; kick sur l'orbite 0
1 orbit "saw" snd c3 note 0.8 comp 0 corbit .  ;; ducker le synthe depuis l'orbite 0
```

## Utiliser Doux depuis d'autres langages

Bob, Boinx et BaLi peuvent envoyer des evenements de notes au slot de Doux.
Doux repond aux messages MIDI note on/off avec sa voix par defaut :

```
DEV 2
>> [note: 60 vel: 100]
WAIT 1
>> [note: 64 vel: 80]
```

Pour un controle complet de la synthese (filtres, effets, FM), utilisez Cagire.

## Mise en route

Doux est active par defaut. Quand vous demarrez le serveur integre depuis
l'application, le moteur audio demarre automatiquement et occupe un slot de
peripherique (verifiez le panneau Peripheriques pour savoir lequel).

1. Ouvrez le panneau Audio, selectionnez votre peripherique de sortie.
2. Verifiez que le moteur tourne.
3. Routez vos evenements vers le slot de Doux.

En session multijoueur, tous les musiciens partagent le meme moteur -- chaque
client peut declencher du son. Si vous n'utilisez que du materiel MIDI externe,
ignorez Doux. Il ne consomme rien quand il est inactif.
