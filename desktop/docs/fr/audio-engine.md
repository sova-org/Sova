# Moteur audio

Sova inclut un moteur audio intégré appelé **Doux**. Il fournit la synthèse et
le traitement audio directement à l'intérieur du serveur, vous permettant de
produire du son sans logiciel ou matériel externe.

## Qu'est-ce que Doux

Doux est un moteur audio temps réel qui fonctionne aux côtés du serveur Sova.
Il occupe un slot de périphérique (généralement le slot 2) et répond aux
événements comme le ferait une sortie MIDI — mais au lieu d'envoyer du MIDI à
un synthé externe, il génère l'audio en interne.

Doux est particulièrement bien intégré au langage **Cagire**, qui dispose de
mots dédiés pour la synthèse audio, la lecture d'échantillons et le traitement
du signal. Les autres langages peuvent envoyer des événements au slot de Doux
pour un déclenchement basique.

## Panneau Audio

Ouvrez le panneau Audio pour configurer le moteur :

- **Périphérique de sortie** — sélectionnez l'interface audio à utiliser pour la
  lecture.
- **Dossiers d'échantillons** — répertoires où Doux cherche les fichiers
  d'échantillons audio.
- **Voix** — le nombre de voix de synthèse simultanées disponibles.

Le panneau Audio affiche aussi l'état du moteur (en cours ou arrêté).

## Panneaux de visualisation

Plusieurs panneaux vous permettent de surveiller la sortie audio en temps réel :

- **Oscilloscope** — affichage de la forme d'onde. Montre le signal audio
  pendant la lecture. Peut être détaché dans une fenêtre séparée.
- **Spectre** — analyseur de spectre fréquentiel. Montre le contenu fréquentiel
  de l'audio. Peut aussi être détaché.
- **VU-mètre** — indicateur de niveau montrant l'amplitude du signal.
- **Barre oscilloscope** — un affichage compact de la forme d'onde qui
  s'intègre dans une barre d'outils.

Ces panneaux reçoivent les données du serveur et se mettent à jour en temps
réel. Ils sont utiles tant pour le monitoring que comme élément visuel pendant
la performance.

## Utiliser Doux depuis le code

Le moyen principal d'utiliser Doux est à travers **Cagire**, le langage à pile.
Cagire fournit des mots pour :

- Les oscillateurs (sinus, dent de scie, carré, triangle, bruit)
- La lecture d'échantillons
- Les filtres et effets
- Les enveloppes d'amplitude
- Le routage du signal

Consultez l'onglet **Cagire** dans la documentation pour la référence complète
des mots de synthèse audio.

Depuis les autres langages (Bob, Boinx, BaLi), vous pouvez envoyer des
événements de notes au slot de Doux. Doux répond aux messages MIDI note on/off
avec sa voix par défaut, vous offrant une synthèse basique sans écrire de code
Cagire.

## Installation

Doux est activé par défaut quand le serveur est compilé avec la fonctionnalité
`audio` (ce qui est le cas dans les builds standards). Quand vous démarrez le
serveur intégré depuis l'application de bureau, le moteur audio est disponible
automatiquement.

Pour utiliser Doux :

1. Ouvrez le panneau Audio et sélectionnez votre périphérique de sortie.
2. Vérifiez que le moteur Doux est en cours d'exécution.
3. Le moteur est assigné à un slot de périphérique (vérifiez le panneau
   Périphériques).
4. Routez vos événements vers ce slot et jouez.

## Astuces

- Doux s'exécute côté serveur. Dans une session multijoueur, tous les musiciens
  partagent le même moteur audio — les événements de n'importe quel client
  peuvent déclencher du son.
- Utilisez les panneaux Oscilloscope et Spectre pendant le sound design pour
  voir ce que votre code de synthèse produit réellement.
- Si vous n'avez pas besoin d'audio intégré (par ex. vous routez du MIDI vers du
  matériel externe), vous pouvez ignorer Doux complètement. Il ne consomme pas
  de ressources si aucun événement ne lui est routé.
