# Control Flow

Sometimes a frame should behave differently depending on context: a coin flip, which iteration of the line is playing. Control flow words let you branch, choose, and repeat inside a frame's script.

## if / else / then

The simplest branch. Push a condition, then `if`:

```forth
coin if 0.8 gain then
saw snd c4 note .
```

The gain is applied if the coin flip is true. The sound always plays. Add `else` for a two-way split:

```forth
coin if
  c4 note
else
  c3 note
then
saw snd 0.6 gain .
```

## ? and !?

When you already have a quotation, `?` executes it if the condition is truthy:

```forth
( 0.4 verb ) coin ?
saw snd c4 note 0.5 gain .    ;; reverb on half the hits
```

`!?` is the opposite. Executes when falsy:

```forth
( 0.2 gain ) coin !?
saw snd c4 note .              ;; quiet on half the hits
```

These pair well with `chance`, `prob`, and the other probability words:

```forth
( 0.5 verb ) 0.3 chance ?      ;; occasional reverb wash
( 12 + ) coin ?                 ;; octave up on coin flip
```

## ifelse

Two quotations, one condition. The true branch comes first:

```forth
( c3 note ) ( c4 note ) coin ifelse
saw snd 0.6 gain .                ;; bass or lead, coin flip
```

Reads naturally: "c3 or c4, depending on the coin."

```forth
( 0.8 gain ) ( 0.3 gain ) coin ifelse
tri snd c4 note 0.2 decay .      ;; loud or quiet, coin flip
```

## select

Choose the nth option from a list of quotations:

```forth
( c4 ) ( e4 ) ( g4 ) ( b4 ) iter 4 mod select
note sine snd 0.5 decay .
```

Four notes cycling through a major seventh chord, one per line iteration. The index is 0-based.

## apply

When you have a quotation and want to execute it unconditionally, use `apply`:

```forth
( dup + ) apply    ;; doubles the top value
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
saw snd 0.6 gain 800 lpf .
```

A different root note each time the line loops.

The last line before `endcase` is the default. It runs when no `of` matched:

```forth
iter 3 mod case
  0 of 0.9 gain endof
  0.4 gain              ;; default: quieter
endcase
saw snd c4 note .
```

## times

Repeat a quotation n times. The variable `@i` is automatically set to the current iteration index (starting from 0):

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
