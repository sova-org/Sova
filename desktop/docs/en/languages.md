No single programming language captures every way of thinking about music. A stack language rewards quick exploration. An imperative language gives precise sequential control. A functional language makes patterns composable. Languages can have a lot of personality or be very personal for experienced live coders too. Sova is built around this observation. Rather than choosing one language, the software provides shared infrastructure (virtual machine, scheduler, [variable](variables) system, [device](devices) routing) and lets multiple languages coexist on top of it. Each language follows its own paradigm and exposes its own abstractions. All of them access the same underlying machinery. See [About Sova](about) for the design philosophy behind this approach.

## Two execution models

Languages integrate with Sova through one of two paths. Both run inside the same VM context (the same [variables](variables), [devices](devices), clock, and [timing](timing)) but they differ in how source code becomes events.

- **Compiled languages**: transform source code into bytecode, a sequence of low-level instructions. The bytecode is stored in the frame. At execution time, the VM's instruction executor steps through it. Compilation happens once; the bytecode runs repeatedly without re-reading the source.

- **Interpreted languages**: skip the bytecode step. They evaluate the source directly using their own logic, but still run inside the same VM context and produce events through the same interface.

From the scheduler's perspective, both models are interchangeable. The distinction matters when implementing a new language. It does not affect what you can express in one.

## The shipped languages

Four languages ship with Sova. They are not a final set but should rather be considered as demonstrations. Each language explores a different perspective in the design space of musical programming languages. Together they cover four distinct paradigms: concatenative, imperative, functional, and declarative.

### Cagire

A concatenative language, inspired by Forth. You push values onto a stack; words consume and produce values on the stack. `.` is a special operator used to emit a command. Cagire includes a lot of words related to music theory, synthesis, etc. Its terseness makes it fast to type for improvisation. Cagire is a good language to use when you are looking to familiarize yourself with the audio engine.

```forth
c4 min7 note
0.5 decay
sine snd
2 vib 0.25 vibmod
.
```

### Bob

An imperative language with Polish notation: operators precede their operands. Events are key-value maps emitted with `>>`. Time advances explicitly when using the `WAIT` function. Bob has variables, conditionals, loops, functions, and rhythm generators. Its explicit control flow gives precise command over event sequencing.

```
RANGE 0 3 :
  >> [note: ADD 60 MUL I 4 vel: 100]
  WAIT 0.25
END
```

### BaLi

BaLi is a declarative language that looks a bit like Lisp and is expression-based. Everything in BaLi looks like an S-expression. Loops, notes, and transformations compose by nesting. Musical patterns are built by composing smaller patterns; the syntax itself encourages algorithmic and generative thinking.

### Boinx

Boinx is a functional language that facilitates musical pattern notation. Sequences `[...]` spread items evenly across the frame. Simultaneous events use `(...)`. Event data goes into hashmaps: `<...>`. Many different operators (`|`, `°`, `~`, `!`, `#`) control how data flows into pattern slots

```
<s: 'kick'> | [. _ . _]
```

## Mixing languages

A single line can hold frames in different languages. Frame 1 might use `Cagire` for a drone, frame 2 `Bob` for a melody, frame 3 `Boinx` for a drum fill. The sequencer plays them in order regardless of language. To change a frame's language, open the editor and press Cmd+L (`Ctrl+L`) or click the language name at the top of the editor window.

## What comes next

Sova is designed so that new languages can be added without modifying the core. A language implementation plugs into the existing infrastructure and immediately gets syntax highlighting, variable access, device routing, and a documentation tab, all through the same interface the shipped languages use. The four shipped languages cover broad paradigms. Future languages can be more specialized: a language designed for a specific instrument, a language that thinks in terms of textures rather than notes, a language built around a performer's personal vocabulary. The only constraint is that the language produces events the scheduler can dispatch. The runtime is stable infrastructure. The languages are where experimentation happens. We hope that Sova will become a runtime in which people can experiment and surprise us.
