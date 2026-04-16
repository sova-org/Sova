# Control Flow

Every programming language comes with control flow mechanisms to create loops, conditions, to write logical processes, etc. Cagire comes with a fairly large set of tools and words that you can use to control the flow of your programs. Some are more useful than others, and some are more _idiomatic_ (forth-like) than others. Pick the ones that make sense to you and go with the flow!

## if / else / then

Push a condition to the stack, then use `if` to branch. This example will sometimes play a note, sometimes not:

```forth
;; coin is producing either 0 (false) or 1 (true)
coin if
tri snd c5 note .
then
```
Add `else` for a two-way split. Here we will have either a very high pitched note or a low pitched one:

```forth
coin if
  c6 note
else
  c3 note
then
saw snd 0.6 gain .1 decay .
```

## ? and !?

This control flow construct plays with quotations. Quotations are fragments of a program that are inactive unless you activate them somehow. `?` and `?!` are able to activate them. Push a quotation to the stack. If some given condition is true, the `?`/`?!` words will execute it:

```forth
;; We have 50/50 chance of adding reverb
( 0.4 verb ) coin ?
saw snd
c4 note
0.5 gain
.
```

`!?` is the opposite. Executes the quotation when falsy:

```forth
( 0.2 gain ) coin !?
saw snd
c4 note
.              ;; quiet on half the hits
```

These pair well with `chance`, `prob` and other probability words:

```forth
( 0.5 verb ) 0.3 chance ?      ;; occasional reverb wash
( 12 + ) coin ?                 ;; octave up on coin flip
```

You will notice that they are used more often than `if`/`then`/`else` and long form counterparts.

## ifelse

Two quotations, one condition. The true branch comes first:

```forth
;; bass or lead, coin flip
( c3 note ) ( c4 note ) coin ifelse
saw snd
5000 500 0.25 slide lpf
0.9 lpq
0.6 gain
.
```

```forth
;; kick or cymbal
( kick snd . )
( cymbal snd . ) coin ifelse
```

## select

Choose the nth option from a list of quotations:

```forth
;; Pick a random note from four different quotations
( c4 ) ( e4 ) ( g4 ) ( b4 ) 0 3 rand select note
sine snd
0.5 decay
2 1 0.1 slide fm
.5 delay .25 delaytime
.
```

## apply

When you have a quotation and want to execute it unconditionally, use `apply`:

```forth
;; this will just unquote 'dup +' which will execute it
( dup + ) apply
```

This is simpler than `?` when there is no condition to check. It pops the quotation and runs it.

## case / of / endof / endcase

For matching a value against several options. Cleaner than a chain of `if`s when you have more than two branches:

```forth
iter 4 mod case
  0 of c3 note endof
  1 of e3 note endof
  2 of g3 note endof
  3 of a3 note endof
endcase
saw snd
0.6 gain
800 lpf 
.
```

A different root note each time the line loops.

The last line before `endcase` is the default. It runs when no `of` matched:

```forth
iter 3 mod case
  0 of 0.9 gain endof
  ;; default: quieter
  0.4 gain
endcase
saw snd c4 note .
```

## times

Repeat a quotation _n_ times. The variable `@i` is automatically set to the current iteration index (starting from 0):

```forth
3 ( c4 @i 4 * + note ) times
sine snd 0.4 gain 0.5 verb .      ;; c4, e4, g#4, a chord
```

Subdivide with `at`:

```forth
4 ( @i 4 / ( sine snd c4 note 0.3 gain . ) at ) times
```

Four evenly spaced notes within the frame.

Vary intensity per iteration:

```forth
8 (
  @i 8 / (
    @i 4 mod 0 = if 0.7 else 0.2 then gain
    tri snd c5 note 0.1 decay .
  ) at
) times
```

Eight notes per frame. Every fourth one louder.

## See also

The words on this page are the explicit branching and looping primitives. Cagire has more ways to conditionally run code, each with its own page:

- [Quotations](quotations.md) for the `( ... )` syntax that `?`, `!?`, `ifelse`, `select`, `apply` and `times` all rely on.
- [Probability](probability.md) for probability-driven execution: `chance`, `prob`, `sometimes`, `rarely`, `often`, `always`, `never`.
- [Periodic](periodic.md) for time-indexed execution: `every`, `except`, `every+`, `except+`, `bjork`, `pbjork`.
- [Generators](generators.md) for `gen` and friends, when you want a quotation to produce a sequence of values.
