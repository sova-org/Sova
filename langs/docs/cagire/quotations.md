# Quotations

A quotation is code in parentheses. It does not run when you write it. It sits on the stack until some other word picks it up and runs it later. Quotations are how Cagire defers code: they let you say "do this *if*", "do this *every n*", "do this *4 times*", "do this *with 25% chance*".

```forth
( kick snd . )
```

That line pushes a quotation onto the stack. Nothing plays. The quotation will only fire if a word like `apply`, `every`, or `chance` picks it up.

## Writing one

Whitespace around `(` and `)` is required.

```forth
( c4 note sine snd . )    ;; quotation
(c4 note sine snd .)      ;; one unknown word, error
```

Anything you can write at top level can go inside: notes, intervals, variables, conditionals, lists, emit, even other quotations.

## Words that take a quotation

Each row below pops one quotation off the stack (sometimes plus extra arguments) and decides when to run it.

| Group | Words |
|---|---|
| Conditional | `?`, `!?`, `ifelse`, `select`, `apply`, `map` |
| Looping | `times` (the iteration index is `@i`) |
| Probability | `chance`, `prob`, `always`, `often`, `sometimes`, `rarely`, `almostAlways`, `almostNever`, `never` |
| Periodic | `every`, `except`, `every+`, `except+`, `bjork`, `pbjork` |
| Sub frame timing | `at` |
| Generation | `gen` |

For periodic firing rules see *Periodic Execution*. For sub frame timing see *Timing*. For probability see *Randomness*.

## Quotations as picked values

Selection words pick one item from a list and push it. If the picked item happens to be a quotation, the word runs it instead of leaving it on the stack. This works for `cycle`, `pcycle`, `bounce`, `pbounce`, `index`, `choose`, `wchoose`.

That gives you the "list of mini phrases" idiom. Each slot is a tiny self contained snippet:

```forth
( c3 note ) ( e3 note ) ( g3 note ) 3 cycle
sine snd .
```

Frame after frame, `cycle` walks through the three quotations and runs the next one. The trailing `sine snd .` plays whichever note the picked quotation set.

See *Cycling* and *Randomness* for the full picker family.

## Scope

A quotation reads and writes the same variables as the code around it. It does not get its own scope. So you can read `@i`, `@beat`, `@runs`, or your own variables from inside, and any `!x` you do inside is visible after.

```forth
4 ( @i 4 * pentatonic note sine snd .2 decay . ) times
```

`@i` here is supplied by `times`. The quotation reads it on every iteration to compute a different note.

## Idioms

A coin flip between two notes, fired only every 4 line iterations:

```forth
( ( c4 note ) ( e4 note ) coin ifelse ) 4 every
```

A hi hat that plays 70% of the time:

```forth
( hat snd 0.4 gain . ) 0.7 chance
```

A run of `n` notes from a scale, indexed by the loop counter:

```forth
4 ( @i 4 * pentatonic note sine snd .2 decay . ) times
```

A two slot probability gate, one of the named shortcuts instead of a literal:

```forth
( crash snd . ) rarely
```

## The mute trick

A quotation no word ever consumes is silently dropped. So wrap any line in `( ... )` to disable it without deleting it:

```forth
( kick snd . )
( hat snd . )
```

The first line is muted. Unwrap it to bring the kick back. Handy in live coding when you want to silence a voice for a few bars without losing the line.

## Gotchas

* Whitespace around `(` and `)`. Forgetting it gives an "unknown word" error, not a quotation.
* Calling a word that does not take a quotation on a quotation gives an "expected quotation" error.
* A quotation is a one shot value, not a stored function. To reuse the same body in many places, write it once and pass it through `cycle`, or define a colon word (see *Creating Words*).
