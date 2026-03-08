# Languages

Sova has four built-in languages. Each frame picks one. You can mix them freely
across frames in the same line -- a Bob melody followed by a Cagire drone
followed by a Boinx drum break.

## Cagire

Stack-based, inspired by Forth. You push values onto a stack and apply words
that consume and produce stack values. `.` emits the current sound command.

A kick drum:

```forth
kick snd .
```

A chord with reverb:

```forth
c4 min7 note 0.4 verb sine snd .
```

A rhythmic pattern using timing offsets and Euclidean distribution:

```forth
3 8 euclid at hat snd .
```

Cagire has built-in music theory -- notes, intervals, chords, scales -- plus
randomness, cycling, variables, and user-defined words. See the **Cagire** tab.

## Bob

Imperative, Polish notation. Operators come before operands: `ADD 2 3` instead
of `2 + 3`. Events are key-value maps emitted with `>>`. Time advances with
`WAIT`.

A four-note sequence:

```
RANGE 0 3 :
  >> [note: ADD 60 MUL I 4 vel: 100]
  WAIT 0.25
END
```

Euclidean rhythm with ghost notes:

```
EU 3 8 0.125 :
  >> [note: 36 vel: 100]
ELSE :
  >> [note: 36 vel: 20]
END
```

Random note selection from a list:

```
SET G.NOTES '[60 64 67 72]
>> [note: PICK G.NOTES vel: RRAND 60 127]
```

Bob has variables (global, frame, line), conditionals, loops, functions, and
Euclidean/binary rhythm generators. See the **Bob** tab.

## BaLi

Lisp-like, expression-based. Everything is an S-expression wrapped in
parentheses. Loops, notes, and effects compose by nesting. Fractions like `1//4`
express durations directly.

A looping note sequence:

```
(loop 4
  (note (+ 60 (* $i 3)) 90)
  1//4)
```

A chord on beat:

```
(note 60 100 dev:1 ch:1)
(note 64 100 dev:1 ch:1)
(note 67 100 dev:1 ch:1)
```

Euclidean rhythm:

```
(eucloop 3 8
  (note 36 100)
  1//8)
```

BaLi's functional style makes it natural for algorithmic composition and
generative patterns. See the **bali** tab.

## Boinx

Declarative pattern notation. You describe *what* plays *where* in time using
brackets and operators. Sequences `[...]` spread items evenly across the frame.
Simultaneous events use `(...)`. Key-value event data goes in `<...>`.

A kick-hat pattern:

```
<s: 'kick'> | [. _ . _]
```

Layered drums with a kick and hi-hat playing simultaneously:

```
(<s: 'kick'> <s: 'hat'>) | [. _ . _]
```

Cycling notes through a rhythmic grid:

```
(C4 E4 G4) ° [. . . .]
```

Boinx operators (`|`, `°`, `~`, `!`, `#`) control how event data flows into
pattern slots. The visual layout of the code mirrors the rhythmic structure.
See the **Boinx** tab.

## Mixing languages

A single line can hold frames in different languages. Frame 1 might be a Cagire
drone, frame 2 a Bob melody, frame 3 a Boinx drum fill. The sequencer plays
them in order regardless of language. To switch a frame's language, open the
editor and pick from the dropdown at the top.
