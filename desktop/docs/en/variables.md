# Variables

Variables store values that persist between events or between frames. Use
them to coordinate scripts, accumulate state, and build evolving patterns.

## Scopes by example

Four scopes. The scope determines who sees the variable and how long it lives.

**Instance** -- local scratch. Resets every time the script runs.

```forth
10 !x @x      ;; store 10, fetch it back
```

**Frame** -- survives repetitions. Resets when the line advances. Good for
counters.

```forth
@F.n 1 + !F.n
@F.n 12 mod note sine snd .   ;; cycles through 12 notes
```

```
SET F.count ADD F.count 1
>> [note: MOD F.count 12]
```

**Line** -- shared across all frames in a line. One frame sets, another reads.

```forth
;; frame A
c4 !L.root
;; frame B
@L.root 7 + note sine snd .
```

```
-- frame A
SET L.root 60
-- frame B
>> [note: ADD L.root 7]
```

**Global** -- visible to every script in the session. Use sparingly.

```forth
c4 !G.key
@G.key note sine snd .
```

```
SET G.key 60
>> [note: G.key]
```

## Store and fetch (Cagire)

`!name` stores the top of the stack. `@name` fetches it. Unknown variables
return 0. `,name` stores and keeps the value on the stack:

```forth
440 ,freq sine snd .   ;; stores 440 AND passes it along
```

Scope prefixes go between operator and name: `!G.x`, `@L.root`, `,F.count`.

## Accumulators

Fetch, modify, store back. Classic pattern for evolving sequences:

```forth
@F.n 1 + !F.n
( 0 !F.n ) @F.n 16 > ?    ;; reset after 16
```

Bob:

```
SET F.n ADD F.n 1
IF GT F.n 16 : SET F.n 0 END
>> [note: ADD 48 MOD F.n 12]
```

## Naming sounds

Store a sound name, reuse across frames:

```forth
;; frame A
"sine" !L.synth
;; frame B, C, D...
c4 note @L.synth snd .
```

Change one frame, all frames in the line follow.

## Environment values

Read-only values from the runtime. The most useful:

- Beat position, tempo, random number
- Frame index, line index, iteration counter

Cagire: `iter` pushes iteration count, `rand` pushes a random value.
Bob: `R` is random 0-127, `I` is loop index, `T` is tempo.

## Visibility timing

Within a frame, you read back what you just wrote. Changes become visible to
other frames only after the current frame finishes. If frame A writes
`10 !G.x` and frame B reads `@G.x` in the same pass, B sees the old value.
