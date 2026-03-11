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
et enregistrement en direct vers des échantillons réutilisables. Cagire offre
l'intégration la plus profonde avec Doux — voir l'onglet Cagire pour la liste
complète des paramètres.

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

## Utiliser Doux

Cagire est le langage principal pour piloter Doux, offrant un contrôle direct
sur chaque paramètre de synthèse : type d'oscillateur, hauteur, gain, fréquence
de coupure, forme d'enveloppe, profondeur FM, effets, orbites et compression
sidechain. Voir l'onglet Cagire pour l'API de synthèse complète.

Bob, Boinx et BaLi peuvent envoyer des événements de notes au slot de Doux.
Doux répond aux messages MIDI Note On/Off avec sa voix par défaut. Pour un
contrôle complet de la synthèse (filtres, effets, FM), utilisez Cagire.

## Enregistrement

Doux peut enregistrer sa propre sortie en échantillons que l'on rejoue et
manipule immédiatement avec des effets. On démarre l'enregistrement, on
l'arrête, et l'audio capturé devient un sample nommé disponible pour tous les
scripts de la session.

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
