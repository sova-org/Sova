# Pour commencer avec Sova

Sova est un séquenceur de live coding. Vous écrivez du code qui génère des
événements musicaux en temps réel — notes, contrôles, messages OSC — et Sova
les joue sur une timeline partagée.

## Concepts

- **Scène** — le conteneur principal. Une scène contient des lignes qui s'exécutent en parallèle.
- **Ligne** — une séquence d'événements temporisés, écrite dans l'un des langages de Sova.
- **Frame** — une cellule dans la grille temporelle. Chaque frame a une durée (en battements) et un nombre de répétitions.
- **Périphérique** — un port MIDI, un point d'accès OSC, ou une sortie audio qui reçoit les événements.

## Écrire votre première séquence

1. Connectez-vous au serveur Sova (ou démarrez le serveur intégré).
2. Sélectionnez une ligne dans la grille de scène.
3. Choisissez un langage (Bob, Boinx, Forth, ou BaLi).
4. Tapez un court programme et appuyez sur **Entrée** pour évaluer.

La ligne commence à produire des événements immédiatement.

## Modes d'exécution

- **Free** — chaque ligne boucle indépendamment à son propre rythme.
- **AtQuantum** — les lignes se resynchronisent à la frontière du quantum global.
- **LongestLine** — toutes les lignes attendent la plus longue avant de redémarrer.
