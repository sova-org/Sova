Cycling steps through values *deterministically*. There is no randomness involved. These words rotate through a list based on an internal counter, so the same script produces the same sequence every time. They are the workhorse of pattern-style live coding: build a list of values, hand it to a cycling word, and let the counter walk through them as the frame triggers.

The two counters that drive cycling come from the runtime context:

- `runs`: how many times *this specific frame* has triggered. Advances per frame.
- `iter`: how many times *this line* has iterated. Advances per line iteration.

Different cycling words read different counters. Choose carefully when your scene has lines with different lengths.

## cycle

`cycle` selects an item by `runs`. Push your values, then their count, then call `cycle`:

```forth
60 64 67 3 cycle note sine snd .
```

That picks `60, 64, 67, 60, 64, 67, ...` over successive frame triggers. Stack effect: `(v1..vn n -- selected)`.

The selected value can be anything that lives on the stack: integers, floats, strings, even quotations. When the chosen item is a quotation, `cycle` evaluates it instead of leaving it on the stack:

```forth
( c4 note ) ( e4 note ) ( g4 note ) 3 cycle
sine snd .
```

This is the standard idiom for "rotate through small subscripts": each cycle iteration runs a self-contained snippet that sets up its own state.

`cycle` advances by `runs`, so two lines that both cycle through `[a b c]` will stay in lockstep only if they trigger at the same rate. Use `pcycle` if you want a counter that ignores frame timing.

## pcycle

`pcycle` is identical to `cycle` except it counts by `iter` (line iterations) instead of `runs` (frame triggers):

```forth
kick snare 2 pcycle snd .    ;; kick on even iterations, snare on odd
```

The difference matters when lines have different durations. A line that triggers four frames per loop will see its `runs` counter advance four times as fast as its `iter` counter, so `cycle` and `pcycle` will diverge. Use `pcycle` when you want a sequence to step "once per pass through the line" regardless of how many frames the line contains.

## bounce / pbounce

`bounce` walks through the list and ping-pongs at the ends instead of wrapping back to the start:

```forth
60 64 67 72 4 bounce note sine snd .
;; runs: 0  1  2  3  4  5  6  7  8 ...
;; pick: 60 64 67 72 67 64 60 64 67 ...
```

`pbounce` does the same with `iter` instead of `runs`:

```forth
60 64 67 72 4 pbounce note sine snd .
```

`bounce` is useful for symmetric patterns: running an LFO-style sweep up and down without explicit phase tracking. The total period is `2*(n-1)`, not `n`, because the endpoints are not duplicated.

## index

`index` selects an item by an *explicit* index that you provide on the stack. The index wraps with modulo, so it never goes out of bounds, and it ignores `runs` and `iter` entirely:

```forth
[ c4 e4 g4 ] step index note sine snd .   ;; step picks the note
[ c4 e4 g4 ] iter index note sine snd .   ;; iteration picks the note
```

Stack effect: `(v1..vn n idx -- selected)`. Combined with context words like `step`, `pattern`, `runs`, or arithmetic, `index` is the escape hatch when you want a counter that none of the other cycling words supplies: picking by a random value, by a tempo-derived integer, or by a hand-rolled accumulator, for example.

## When Quotations Get Executed

The cycling family treats quotations as executable bodies, not as data. If your value list happens to contain a quotation in some slots and plain data in others, only the quotation slots execute. The plain data is left on the stack as usual:

```forth
60 ( e4 note ) 67 3 cycle    ;; runs 0: pushes 60, runs 1: runs (e4 note), runs 2: pushes 67
```

This is identical to how `choose` (in [Randomness](#)) handles quotations, so the same idioms transfer.

## Combining With Other Words

Cycling pairs naturally with `at`, `every`, and the chord/scale system. A four-step bassline that doubles every other bar:

```forth
c2 ( 0 ) ( 7 ) ( 5 ) ( 12 ) 4 cycle +
( 0 ) ( -12 ) 2 pcycle +
note 0.8 gain sine snd .
```

Stepping a chord progression with `pcycle`:

```forth
( c3 note maj7 chord . )
( f3 note maj7 chord . )
( g3 note 7 chord . )
( c3 note maj7 chord . )
4 pcycle
```

Switching scales mid-piece using `index` driven by `pattern`:

```forth
major minor dorian 3 pattern index
c4 swap [ 0 2 4 7 ] cycle deg freq
sine snd .
```

For periodic firing (run a quotation *every nth iteration*) rather than picking from a list, see the [Periodic Execution](#) article. For the wider framing of cycling as a way to modulate parameters, see [Control Rate Modulation](#).
