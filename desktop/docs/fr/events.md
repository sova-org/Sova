# Événements

Votre code produit des événements : messages MIDI, messages OSC ou commandes
audio envoyées aux périphériques. Chaque événement porte des paramètres —
hauteur, vélocité, durée, canal, slot — qui déterminent ce qui sonne et où.

## Notes MIDI

Un événement de note envoie un Note On, puis un Note Off quand la durée est
écoulée. On n'envoie jamais de Note Off soi-même — le moteur s'en charge.

Paramètres : hauteur (0–127), vélocité (0–127), durée (beats), canal (1–16),
périphérique (1–16). Par défaut : vélocité 100, durée 0.5 beats, canal 1,
périphérique 1.

## Control Change

Les CC contrôlent les potentiomètres, faders et paramètres sur les
synthétiseurs ou DAW externes. On spécifie un numéro de CC (0–127) et une
valeur (0–127). Le canal et le périphérique suivent le même routage que les
notes.

## Pitch bend

Une valeur continue de -1.0 (fond bas) à 1.0 (fond haut), centre à 0.0.
Envoyée sur le canal et le périphérique courants.

## Program Change

Sélectionne un patch ou un preset sur le périphérique cible. Un simple numéro
(0–127).

## Messages OSC

OSC envoie des messages par UDP vers SuperCollider, Max/MSP, Pure Data ou toute
application compatible. Un événement OSC porte un chemin d'adresse et un
ensemble d'arguments clé-valeur. Routez-le vers un slot OSC pour atteindre la
bonne application.

## Routage périphérique et canal

Chaque événement porte un slot de périphérique et un canal MIDI. Le
périphérique sélectionne le slot de sortie (1–16). Le canal sélectionne le
canal MIDI au sein de ce périphérique. Le slot 0 est la console de log — on
l'utilise pour inspecter les événements avant de les router vers une sortie
réelle. On peut changer de périphérique et de canal en cours de script, en
envoyant différents événements vers différentes destinations depuis la même
frame. Voir **Périphériques** pour la configuration des slots.

## Accords et séquences

Sans timing explicite, tous les événements d'un script se déclenchent en même
temps — produisant des accords ou des sons superposés. Pour espacer les
événements dans le temps, utilisez le mécanisme de timing de votre langage :
attentes, décalages ou arpégiation. Voir **Timing** pour les détails.

## Lire l'entrée MIDI

Les valeurs CC entrantes depuis des contrôleurs matériels peuvent être lues
dans vos scripts. Cela permet à des potentiomètres et faders physiques de
piloter des paramètres en temps réel — fréquences de coupure, profondeurs
d'effets, transpositions. Connectez un périphérique d'entrée MIDI dans le
panneau Périphériques et utilisez les fonctions d'entrée du langage pour lire
les valeurs.

Voir les onglets de langage pour la syntaxe de chaque type d'événement.
