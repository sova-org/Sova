# Lists

Square brackets collect values and push their count. `[ 60 64 67 ]` is exactly the same as typing `60 64 67 3`. Most variable arity words in Cagire expect that count on top of the stack, so brackets save you from counting items by hand.

```forth
60 64 67 3 cycle      ;; counted by hand
[ 60 64 67 ] cycle    ;; counted for you
```

Whitespace around `[` and `]` is required, same as for quotations.

## Anything goes inside

Numbers, note names, intervals, variables, arithmetic, nested brackets, even quotations.

```forth
[ c4 c4 m3 + c4 P5 + ] note sine snd .   ;; minor triad via interval words
[ @low @high rand ] note sine snd .       ;; one random note in a range
```

The brackets compile and run their contents normally. Whatever ends up on the stack between `[` and `]` is what gets counted.

## Words that pair with `[ ]`

Any word with a stack signature like `(v1..vn n -- ...)` is happy to receive a bracketed list.

| Group | Words | One liner |
|---|---|---|
| Cycling | `cycle`, `pcycle`, `bounce`, `pbounce`, `index` | `[ c4 e4 g4 ] cycle note sine snd .` |
| Random selection | `choose`, `shuffle` | `[ kick snare hat ] choose snd .` |
| Sub frame timing | `at` | `[ 0 0.25 0.5 0.75 ] ( hat snd . ) at` |
| Polyphonic params | `note`, `freq`, `gain`, `n`, etc. | `[ 60 64 67 ] note sine snd .` |
| Stack utilities | `dupn`, `rev`, `sort`, `rsort`, `sum`, `prod` | `[ 5 2 9 1 ] sort` |

The polyphonic case is worth highlighting: if you give a parameter word more than one value, the next emit fans out into one event per value. So `[ 60 64 67 ] note sine snd .` plays a three note chord with no chord quality system involved at all.

For the full picker family see *Cycling* and *Randomness*. For `at` deltas see *Timing*.

## A note on `wchoose`

`wchoose` consumes value/weight *pairs*, and the count it expects is the number of pairs, not the number of items. So a bracketed list does not line up cleanly: `[ kick 0.5 snare 0.3 hat 0.2 ]` would push count `6`, but `wchoose` wants `3`. Write it the manual way:

```forth
kick 0.5 snare 0.3 hat 0.2 3 wchoose snd .
```

## Nesting

Brackets nest. Each pair produces its own count, independent of the others.

```forth
[ [ c4 e4 g4 ] cycle [ 0.3 0.5 0.8 ] cycle ] choose
```

The inner cycles each pick one item per frame. The outer brackets then choose between the two picked values.

## Curly braces

There is one more bracket form: `{ }`. It is stripped before compilation and produces no code at all. Pure visual grouping for dense lines:

```forth
{ kick snd } { 0.5 gain } { 0.3 verb } .
```

That compiles to exactly `kick snd 0.5 gain 0.3 verb .`. Use it when a long parameter chain is hard to read at a glance.
