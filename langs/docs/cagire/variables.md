# Variables

Variables let you name values and share data between frames. They are global — any frame in any line can read what another frame wrote.

## Store and Fetch

`!name` stores the top of the stack into a variable. `@name` fetches it back. Variables spring into existence when you first store to them. Fetching a variable that was never stored returns 0.

```forth
10 !x        ;; store 10 in x
@x           ;; pushes 10
@y           ;; pushes 0 (never stored)
```

## Store and Keep

`,name` stores just like `!name` but keeps the value on the stack. Useful when you want to name something and keep using it:

```forth
440 ,freq sine snd .   ;; stores 440 in freq AND passes it to the pipeline
```

Without `,`, you'd need `dup`:

```forth
440 dup !freq sine snd .   ;; equivalent, but noisier
```

## Sharing Between Frames

Variables are shared across all frames in all lines. One frame can store a value that another reads:

```forth
;; frame A: pick a root note
c4 iter 7 mod + !root

;; frame B: read it
@root 7 + note sine snd .
```

Every time the line loops, frame A picks a new root. Frame B always harmonizes with it.

## Accumulators

Fetch, modify, store back. A classic pattern for evolving values:

```forth
@n 1 + !n              ;; increment n each time this frame runs
@n 12 mod note sine snd . ;; cycle through 12 notes
```

Reset on some condition:

```forth
@n 1 + !n
( 0 !n ) @n 16 > ?    ;; reset after 16
```

## When Changes Take Effect

Within a single frame, you can read back what you just wrote. But variable changes only become visible to other frames after the current frame finishes executing. If frame A writes `10 !x` and frame B reads `@x` in the same scheduling pass, frame B sees the value from the previous pass, not the current one.

## Naming Sounds

Store a sound name in a variable, reuse it across frames:

```forth
;; frame A: choose the sound
"sine" !synth

;; frame B, C, D...
c4 note @synth snd .
```

Change one frame, all frames follow.
