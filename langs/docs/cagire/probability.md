Probability words take a quotation and run it with some chance. They are how you turn *maybe* into a musical decision: "play this hat 25% of the time", "add reverb occasionally", "mute this voice for now". The quotation sits deeper on the stack, the probability sits on top.

For deterministic firing, see the [Periodic Execution](#) article. For deterministic stepping through values, see [Cycling](#). For the general `if`/`?`/`ifelse` primitives, see [Control Flow](#).

## chance and prob

`chance` takes a float from 0.0 to 1.0. `prob` takes a percentage from 0 to 100. Both pop the quotation below the probability and run it with that chance:

```forth
( hat snd . ) 0.25 chance    ;; 25% chance
( hat snd . ) 75 prob        ;; 75% chance
```

Stack effect: `(quot prob --)` for both.

Use them on anything that takes a quotation as its "thing to do":

```forth
kick snd
( 0.4 verb ) 0.3 chance    ;; occasional reverb wash
.
```

```forth
( 12 + ) 20 prob           ;; octave up 20% of the time
c4 note saw snd .
```

## Named probabilities

Named probability words are shorthand for the most common cases. They save you from typing `0.5 chance` when `sometimes` reads better:

| Word | Probability |
|------|------------|
| `always` | 100% |
| `almostAlways` | 90% |
| `often` | 75% |
| `sometimes` | 50% |
| `rarely` | 25% |
| `almostNever` | 10% |
| `never` | 0% |

```forth
( hat snd . ) often          ;; 75%
( snare snd . ) sometimes    ;; 50%
( clap snd . ) rarely        ;; 25%
```

`always` and `never` look pointless at first glance, but they earn their keep as live-coding mute switches: write a voice with `always`, change it to `never` to silence the voice without deleting the line, then change it back when you want it.

```forth
( kick snd . ) always       ;; on
( snare snd . ) never       ;; muted, still in the code
( hat snd . ) often
```

## Coin-flip shorthand

`coin` plus `?` or `!?` gives a quick coin-flip gate:

```forth
( hat snd . ) coin ?     ;; execute if coin is 1
( rim snd . ) coin !?    ;; execute if coin is 0
```

`?` runs its quotation when the value above it is truthy, `!?` runs it when falsy. They are general-purpose conditionals covered in [Control Flow](#), but they pair very naturally with `coin` for one-line 50/50 variation. The pair `coin ?` and `coin !?` on two lines gives you two alternatives, one of which fires per trigger:

```forth
( kick snd . ) coin ?
( snare snd . ) coin !?
```

## See also

- [Randomness](#) for `coin`, `rand`, `choose`, `wchoose`, `shuffle`, and `seed`: the words that return values instead of running quotations.
- [Control Flow](#) for `if`/`else`/`then`, `ifelse`, `select`, `apply`, and the general `?`/`!?` behavior.
- [Periodic Execution](#) for `every`, `except`, `bjork` and friends: the deterministic counterparts to these probability gates.
