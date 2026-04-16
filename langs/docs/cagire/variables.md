# Variables

Variables let you name values and reuse them across frames. They come in four scopes, controlled by an optional prefix on the variable name.

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

## Scopes

By default, variables are **Instance**-scoped: local to the current script execution. Other scripts cannot see them. To share data, use a scope prefix:

| Prefix | Scope | Visible to |
|--------|-------|------------|
| *(none)* | Instance | This script only |
| `G.` | Global | All scripts in the session |
| `L.` | Line | All frames in the same line |
| `F.` | Frame | Persists across runs of the same frame |

The prefix goes between the operator and the variable name:

```forth
10 !G.x      ;; store 10 in global x
@G.x         ;; fetch global x from any script
10 ,G.x      ;; store-and-keep, global

10 !L.root   ;; store 10 in line-scoped root
@L.root      ;; fetch from any frame in this line

10 !F.count  ;; store in frame-scoped count
@F.count     ;; persists between runs of this frame
```

### When to use each scope

**Instance** (default). Scratch values within a single script. Safe, no side effects. Resets each time the script runs.

```forth
@i 12 mod note sine snd .   ;; @i is the loop counter, instance-scoped
```

**Frame** (`F.`). State that accumulates across runs of the same frame. Useful for counters and evolving values.

```forth
@F.n 1 + !F.n              ;; increment each time this frame runs
@F.n 12 mod note sine snd .
```

**Line** (`L.`). Share data between frames in the same line. One frame can set a value that another reads.

```forth
;; frame A: pick a root note
c4 iter 7 mod + !L.root

;; frame B: harmonize
@L.root 7 + note sine snd .
```

**Global** (`G.`). Share data across all scripts in the entire session. Use sparingly: any script can overwrite a global.

```forth
;; any script can set the key
c4 !G.key

;; any other script can read it
@G.key note sine snd .
```

## Accumulators

Fetch, modify, store back. A classic pattern for evolving values. Use Frame scope so the counter persists:

```forth
@F.n 1 + !F.n                ;; increment n each time this frame runs
@F.n 12 mod note sine snd .  ;; cycle through 12 notes
```

Reset on some condition:

```forth
@F.n 1 + !F.n
( 0 !F.n ) @F.n 16 > ?    ;; reset after 16
```

## When Changes Take Effect

Within a single frame, you can read back what you just wrote. But variable changes only become visible to other frames after the current frame finishes executing. If frame A writes `10 !G.x` and frame B reads `@G.x` in the same scheduling pass, frame B sees the value from the previous pass, not the current one.

## Naming Sounds

Store a sound name in a variable, reuse it across frames:

```forth
;; frame A: choose the sound
sine !L.synth

;; frame B, C, D...
c4 note @L.synth snd .
```

Change one frame, all frames in the line follow.
