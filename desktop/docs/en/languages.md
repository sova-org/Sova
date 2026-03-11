# Languages

Sova is a *polyglot* environment. Multiple languages coexist within the same
virtual machine, sharing the scheduler and I/O layer. Each language follows its
own paradigm and exposes different abstractions for the musician to work with.
This diversity fosters experimentation: pick the language that best fits the
musical idea at hand.

Two compilation models coexist. Bob and BaLi compile to bytecode executed by the VM — their scripts can be paused and resumed mid-execution. Boinx and Cagire are interpreted — they evaluate the source directly and emit events immediately.

Four languages ship with Sova. Each frame chooses one. They can be mixed freely
across frames in the same line — a Bob melody followed by a Cagire drone, then
a Boinx rhythmic transition.

## Cagire

Stack-based, inspired by Forth. You push values onto a stack and apply words
that consume and produce stack values. `.` emits the current sound command.

```forth
c4 min7 note 0.4 verb sine snd .
```

Cagire has built-in music theory — notes, intervals, chords, scales — plus
randomness, cycling, variables, and user-defined words. See the **Cagire** tab.

## Bob

Imperative, Polish notation. Operators come before operands. Events are
key-value maps emitted with `>>`. Time advances with `WAIT`.

```
RANGE 0 3 :
  >> [note: ADD 60 MUL I 4 vel: 100]
  WAIT 0.25
END
```

Bob has variables (global, frame, line), conditionals, loops, functions, and
Euclidean/binary rhythm generators. See the **Bob** tab.

## BaLi

Lisp-like, expression-based. Everything is an S-expression wrapped in
parentheses. Loops, notes, and effects compose by nesting. Fractions like `1//4`
express durations directly.

```
(loop 4
  (note (+ 60 (* $i 3)) 90)
  1//4)
```

BaLi's functional style makes it natural for algorithmic composition and
generative patterns. See the **BaLi** tab.

## Boinx

Declarative pattern notation. You describe *what* plays *where* in time using
brackets and operators. Sequences `[...]` spread items evenly across the frame.
Simultaneous events use `(...)`. Key-value event data goes in `<...>`.

```
<s: 'kick'> | [. _ . _]
```

Boinx operators (`|`, `°`, `~`, `!`, `#`) control how event data flows into
pattern slots. The visual layout of the code mirrors the rhythmic structure.
See the **Boinx** tab.

## Mixing languages

A single line can hold frames in different languages. Frame 1 might be a Cagire
drone, frame 2 a Bob melody, frame 3 a Boinx drum fill. The sequencer plays
them in order regardless of language. To switch a frame's language, open the
editor and pick from the dropdown at the top.
