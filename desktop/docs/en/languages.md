# Languages

Sova is polyglot — each frame can use a different programming language. The four
built-in languages offer distinct approaches to musical expression. Pick the one
that fits how you want to think about your music, or mix them freely.

## Compiled vs interpreted

Sova languages fall into two categories:

- **Compiled languages** (Bob, BaLi) are translated into bytecode for Sova's
  virtual machine. The VM executes the bytecode each time the frame plays.
  Compilation happens once when you evaluate; execution is fast and repeatable.
- **Interpreted languages** (Boinx, Cagire) produce a list of events directly
  from the source code each time the frame plays. There's no intermediate
  bytecode step.

From a user perspective, both work the same way: write code, evaluate, hear
results. The difference matters when you want to understand how your code
interacts with variables, timing, and repetitions.

## Overview

- **Bob** — Compiled. Imperative, event maps. Best for melodic sequences, precise control.
- **BaLi** — Compiled. Expression-based, functional. Best for algorithmic patterns, math-heavy.
- **Boinx** — Interpreted. Pattern notation. Best for quick rhythmic patterns.
- **Cagire** — Interpreted. Stack-based (Forth-like). Best for audio synthesis, DSP, experimentation.

## Bob

Bob is an imperative language with a concise syntax for generating MIDI and OSC
events. It uses **event maps** — key-value structures that describe notes,
control changes, and other messages. Bob has variables, conditionals, loops, and
functions.

```
>> [note: 60 vel: 100 dur: 0.5]
WAIT 0.5
>> [note: 64 vel: 80 dur: 0.5]
```

See the **Bob** tab for the full reference.

## BaLi

BaLi is an expression-based compiled language with a functional flavor. It
emphasizes composing transformations and works well for algorithmic,
generative patterns.

See the **BaLi** tab for the full reference.

## Boinx

Boinx is a pattern notation language — its syntax is designed for writing
rhythmic sequences quickly. Patterns describe when events fire within a beat
or a bar, making it natural for drum patterns and percussive sequences.

See the **Boinx** tab for the full reference.

## Cagire

Cagire is a stack-based language inspired by Forth. You push values onto a
stack and apply words (operations) to them. Cagire is tightly integrated with
the Doux audio engine for real-time sound synthesis and DSP, but it also works
for MIDI and OSC output.

See the **Cagire** tab for the full reference.

## Switching languages

Each frame has its own language setting. To change a frame's language:

1. Open the step editor (double-click a frame cell).
2. Select the language from the dropdown at the top of the editor.
3. Write or rewrite your code in the new language.
4. Evaluate.

Different frames in the same line can use different languages — Sova doesn't
care. A line might have a Bob frame generating melodies followed by a Boinx
frame for a drum break. Mix and match as you see fit.
