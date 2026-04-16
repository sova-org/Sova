# Cagire Contributor Guide

This file applies only within `langs/src/cagire/`. Follow the root `AGENTS.md` first, then `langs/AGENTS.md`, then use this guide for Cagire-specific design, architecture, and editing constraints.

## Purpose And Design North Star

Cagire is not trying to be a general-purpose Forth clone. It is a music-first, terse, efficient stack language for live coding inside Sova.

Its design center is:

- minimal syntax
- short programs with strong musical payoff
- postfix composition over expression syntax
- domain words over user ceremony
- immediate translation from a tiny script to scheduled musical events

When evaluating a change, prefer the option that keeps Cagire fast to type, easy to reshape during playback, and easy to reason about as stack flow.

Good Cagire features usually look like:

- a small word that composes with existing words
- a musical operation expressed as stack transformation
- sugar that removes counting or repetition without obscuring execution
- runtime behavior that stays predictable under repeated frame execution

Bad fits usually look like:

- bespoke syntax for a narrow special case
- features that require users to stop thinking in stack effects
- abstractions that hide event creation or timing behind too much machinery
- changes that make live edits harder to predict on the next frame

## Mental Model

Contributors should think of Cagire as five layers working together:

- source text with very little syntax
- a compiler that lowers tokens to `Op`
- a VM that executes `Op` against a stack and command register
- an interpreter that yields scheduled events over time
- Sova runtime dispatch that turns those events into sound, MIDI, OSC, print output, or state changes

The core user-facing model is:

- values go onto a stack
- words consume and produce stack values
- sound-building words populate a command register
- `.` emits the current command as one or more `ConcreteEvent`s
- scripts run once per frame evaluation and may schedule multiple events within that frame

Important semantics to preserve:

- The stack is the primary programming model. Variables, quotations, and bracket forms support the stack model; they do not replace it.
- The command register is Cagire's equivalent of "output". Scripts do not talk directly to an audio engine.
- Quotations are first-class code values used for control flow, timing, selection, and repetition.
- Square brackets are immediate execution plus count sugar. They are not lists in the conventional language-design sense.
- Unknown words intentionally become strings. This is how terse sound names and parameter values stay ergonomic.
- Variables are scope-sensitive runtime state, not mere compile-time names.

## Architecture Map

Read these files together before making non-trivial changes.

- `mod.rs`
  Module assembly only. Do not expect semantics here.

- `compiler.rs`
  Tokenizes source and lowers it into `Vec<Op>` plus matching `Span`s. This file owns the language's small amount of syntax:
  numbers, strings, comments, quotations, brackets, `: ... ;`, `if/else/then`, `case/of/endof/endcase`, and `pat`.

- `words/mod.rs`
  Aggregates the word catalog. `WORDS` metadata is part of the language surface, not docs garnish.

- `words/compile.rs`
  Maps names to compile behavior. This is where built-in words, variable prefixes, note-name parsing, interval parsing, dictionary expansion, and the "unknown words become strings" rule come together.

- `words/core.rs`, `sound.rs`, `effects.rs`, `sequencing.rs`, `music.rs`, `midi.rs`, `osc.rs`
  Own the word metadata grouped by domain. If a word exists publicly, its metadata should usually live here even if the runtime implementation is elsewhere.

- `ops.rs`
  Defines the VM instruction set. Adding an `Op` is a language-design change, not just a refactor detail.

- `types.rs`
  Defines `Value`, `Span`, `CagireError`, `Stack`, resolved-value annotations, and `CmdRegister`. This file explains a lot about what the runtime can and cannot represent.

- `vm.rs`
  The semantic core. Executes `Op`s, reads evaluation context, manages variable interaction, handles pattern timing, expands polyphony, and produces `ConcreteEvent`s. Most behavioral changes eventually land here.

- `interpreter.rs`
  Wraps VM evaluation in Sova's interpreter interface. This file owns step-wise event delivery, wait gaps, dictionary sharing across frames, and editor annotations.

- `factory.rs`
  Binds Cagire into Sova. It exposes documentation, syntax-highlighting rules, and the shared dictionary container. If docs/reference/syntax appear out of sync with runtime behavior, inspect this file.

- `pattern.rs`
  Owns the mini pattern grammar used by `pat` and string-pattern timing with `at`. Treat it as part of the language, not an isolated parser utility.

## Architectural Invariants

These rules are part of the current design and should not be changed casually.

- Unknown words must remain intentional string literals unless the language design is explicitly changing. This is a core terseness feature.
- Word definitions created with `: ... ;` are session-shared through the factory/interpreter dictionary. Changing that changes the language model, not just implementation.
- `Span` tracking is not optional. Compiler spans feed error locations, resolved-value inserts, selected-hit highlights, and current-event highlighting in the editor.
- `at` is quotation-driven and re-executes per delta. Random or cycling behavior inside that quotation must behave per subdivision, not as a one-time precomputed result, unless a deliberate semantic change is being made.
- Parameter words are batch consumers. They often read multiple stack values and expand them into polyphonic event parameters. Do not "simplify" this into single-value semantics without treating it as a language redesign.
- Variable scopes are semantic:
  unprefixed instance-local state,
  `G.` global,
  `L.` line,
  `F.` frame.
  Changes to scope behavior affect cross-frame composition and should be treated carefully.
- Curly braces are ignored syntax. Parentheses and brackets are meaningful. Do not blur these forms.
- Cagire is intentionally permissive at parse time and specific at execution time. Keep that bias unless there is a compelling live-coding reason not to.

## Public Docs And Implementation

The articles under `langs/docs/cagire/` are the public language docs consumed through `factory.rs`. They describe how users experience Cagire.

This `AGENTS.md` is the implementation companion. It should help contributors answer:

- where a behavior lives
- what else must change when a word changes
- which semantics are deliberate rather than accidental
- how to preserve the language's terse live-coding feel

When editing Cagire, keep these aligned:

- runtime behavior in `vm.rs` and related code
- compile behavior in `compiler.rs` or `words/compile.rs`
- word metadata in `words/*`
- examples under `langs/docs/cagire/examples/`
- article or reference content surfaced by `factory.rs`
- syntax metadata in `factory.rs` if token classes changed

## How To Add Or Change Features

### Add A New Simple Word

Use this path when the word compiles directly to an existing `Op` or a new single-purpose `Op`.

- Add or update the word metadata in the appropriate `words/*.rs` module.
- Register the compile mapping in `words/compile.rs`.
- If needed, add a new `Op` in `ops.rs`.
- Implement the runtime semantics in `vm.rs`.
- Add or update examples and tests.

Prefer adding a small composable word over inventing new syntax.

### Add A Parameter, Context, Or Probability Word

If the word is just metadata-driven:

- add it to the right `WORDS` slice with the correct category, stack effect, aliases, example, and `WordCompile` mode
- let `words/compile.rs` lower it through `WordCompile::Param`, `Context`, or `Probability`
- update tests only where behavior or highlighting needs coverage

Be careful with categories. `factory.rs` uses categories to build syntax-highlighting buckets and reference docs.

### Add Compile-Time Sugar

If the feature changes token lowering rather than runtime semantics:

- start in `compiler.rs` if it is real syntax
- start in `words/compile.rs` if it is a name-level sugar or alias-like transformation

Prefer sugar that preserves the existing mental model. Good examples already in-tree:

- shorthand float normalization
- note and interval names
- bracket count sugar
- convenience words like `linramp`, `expramp`, and `logramp`

Avoid syntax that introduces hidden control flow or weakens stack transparency.

### Change Runtime Behavior

If semantics change during execution:

- inspect `vm.rs` first
- trace how stack values, spans, command register state, and evaluation context interact
- verify whether the behavior also affects annotations in `interpreter.rs`
- check whether the change alters how docs describe the language

Runtime changes frequently have cross-effects in:

- polyphony
- timing offsets
- variable visibility
- dictionary sharing
- editor feedback

### Extend Pattern Syntax

Pattern work belongs in `pattern.rs`, but do not treat it as isolated. Pattern timing affects:

- `pat` compilation validation in `compiler.rs`
- event scheduling and highlight spans in `vm.rs`
- interpreter annotations in `interpreter.rs`
- tests in both `pattern.rs` and the timing/highlight tests elsewhere

Pattern syntax should remain concise and rhythm-oriented. Avoid turning it into a second full language.

## Design Guidance For Future Changes

When in doubt, optimize for live-coding ergonomics, not language-theory completeness.

- Prefer one short word over one more structural form.
- Prefer stack-flow composition over named-argument or expression-heavy designs.
- Prefer orthogonal musical primitives that combine well over highly specialized megawords.
- Prefer explicit emitted behavior over hidden side effects.
- Prefer features that are easy to insert, delete, or reorder during playback.

Be skeptical of changes that:

- add syntax users must memorize before they can improvise
- make the next frame's behavior harder to predict
- reduce annotation fidelity or error locality
- make timing depend on incidental implementation details
- split the public word surface from `WORDS` metadata
- change shared dictionary or variable-scope behavior without explicit intent

If a proposed improvement makes Cagire more "normal" as a programming language but less terse and musical, it is probably the wrong direction.

## Validation Guidance

Use the smallest relevant validation first.

- `compiler.rs` tests cover tokenization, spans, quotations, brackets, control-flow lowering, and basic syntax invariants.
- `vm.rs` tests are the main semantic surface: arithmetic, word definitions, variables, emit behavior, MIDI/OSC/generic events, `at`, cycling, randomness, and pattern-linked highlighting.
- `interpreter.rs` tests cover step-wise event delivery and annotation progression.
- `pattern.rs` tests cover the pattern mini-language directly.
- `factory.rs` tests cover syntax-highlighting classification and documentation-facing buckets.

For most Cagire changes, start with the smallest relevant command:

```bash
cargo test -p langs cagire
```

Then widen to:

```bash
cargo test -p langs
```

For non-trivial Rust changes in this subsystem, also run:

```bash
cargo clippy -p langs
```

Docs-only changes in this directory do not require a build unless you changed technical claims and want to spot-check them against the current implementation.

## Change Checklist

Before considering a Cagire change complete, check:

- implementation updated in the right layer
- `WORDS` metadata, aliases, categories, stack effect, and example kept in sync
- syntax highlighting or docs registration updated if the public surface changed
- targeted tests added or updated near the changed behavior
- behavior still feels terse, composable, and live-coding friendly

## Editing Heuristics

If you are not sure where to start:

- syntax or structural forms: `compiler.rs`
- word mapping and sugar: `words/compile.rs`
- public word catalog and reference metadata: `words/*`
- VM semantics and event generation: `vm.rs`
- step/wait/annotation behavior: `interpreter.rs`
- public docs and highlighting surface: `factory.rs`
- pattern mini-language: `pattern.rs`

Update the smallest number of files that preserves these boundaries.
