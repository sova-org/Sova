# Cagire vs Classic Forth

Cagire is not a classic Forth. It borrows the core ideas (stack-based evaluation, postfix notation, word definitions) but adds modern features and domain-specific extensions. If you know traditional Forth, here are the differences.

## Comments

Classic Forth uses parentheses for comments. In Cagire, parentheses are quotation syntax (see below), so they cannot be used for comments.

Cagire uses double semicolons for comments:

```forth
;; this is a comment
```

Everything after `;;` until the end of the line is ignored. Curly braces `{ }` are silently ignored — they have no effect and can be used as visual separators if you like, but they carry no semantic meaning.

## Quotations

Classic Forth has no quotations. Code is not a value you can pass around.

Cagire has first-class quotations using parentheses:

```forth
( dup + )
```

This pushes a block of code onto the stack. You can store it, pass it to other words, and execute it later. Quotations enable conditionals, probability, and cycling.

## Conditionals

Classic Forth uses `IF ... ELSE ... THEN`:

```forth
x 0 > IF 1 ELSE -1 THEN
```

Cagire supports this syntax but also provides quotation-based conditionals:

```forth
( 1 ) ( -1 ) x 0 > ifelse
```

The words `?` and `!?` execute a quotation based on a condition:

```forth
( "kick" snd . ) coin ?     ;; execute if coin is 1
( "snare" snd . ) coin !?   ;; execute if coin is 0
```

## Strings

Classic Forth has limited string support. Cagire has first-class strings:

```forth
"hello"
```

This pushes a string value onto the stack. Strings are used for sound names, sample names, and variable keys. You often do not need quotes at all. Any unrecognized word becomes a string automatically:

```forth
kick snd .       ;; "kick" is not a word, so it becomes the string "kick"
myweirdname    ;; pushes "myweirdname" onto the stack
```

This makes scripts cleaner. You only need quotes when the string contains spaces or conflicts with a real word.

## Variables

Classic Forth declares variables explicitly:

```forth
VARIABLE x
10 x !
x @
```

Cagire uses prefix syntax:

```forth
10 !x    ;; store 10 in x
@x       ;; fetch x (returns 0 if undefined)
10 ,x    ;; store 10 in x, keep on stack
```

No declaration needed. Variables spring into existence when you store to them. `,x` stores and keeps the value on the stack.

## Floating Point

Classic Forth (in its original form) has no floating point. Cagire has native floating point:

```forth
3.14159
0.5 0.3 +    ;; 0.8
```

Integers and floats mix freely. Division always produces a float.

## Loops

Classic Forth has `DO ... LOOP`:

```forth
10 0 DO I . LOOP
```

Cagire uses a quotation-based loop with `times`:

```forth
4 ( @i . ) times    ;; prints 0 1 2 3
```

The loop counter is stored in the variable `i`, accessed with `@i`.

```forth
4 ( @i 4 / at hat snd . ) times    ;; hat at 0, 0.25, 0.5, 0.75
4 ( c4 @i + note sine snd . ) times ;; ascending notes
```

For generating sequences without side effects, use `..` or `gen`:

```forth
1 5 ..          ;; pushes 1 2 3 4 5
( dup * ) 4 gen ;; pushes 0 1 4 9 (squares)
```

## The Command Register

This is completely unique to Cagire. Traditional Forth programs print text. Cagire programs build sound commands.

The command register accumulates a sound name and parameters:

```forth
"sine" sound    ;; set sound
440 freq        ;; add parameter
0.5 gain        ;; add parameter
.               ;; emit and clear
```

Nothing is sent until you emit with `.`.

## Context Words

Cagire provides words that read the current sequencer state:

| Word | Description |
|------|-------------|
| `step` | Current frame index |
| `beat` | Current beat position |
| `pattern` | Current line index |
| `tempo` | Current BPM |
| `phase` | Phase in bar (0.0-1.0) |
| `slot` | Current line index |
| `runs` | Times this frame has triggered |
| `iter` | Line iteration count |
| `stepdur` | Frame duration in seconds |

These have no equivalent in classic Forth. They connect your script to the sequencer's timeline.

## Probability

Classic Forth is deterministic. Cagire has built-in randomness:

```forth
( "snare" snd . ) 50 prob       ;; 50% chance
( "clap" snd . ) 0.25 chance    ;; 25% chance
( "hat" snd . ) often           ;; 75% chance
( "rim" snd . ) sometimes       ;; 50% chance
( "tom" snd . ) rarely          ;; 25% chance
```

These words take a quotation and execute it probabilistically.

## Periodic Execution

Execute a quotation on specific iterations:

```forth
( "snare" snd . ) 4 every        ;; every 4th line iteration
( "hat" snd . ) 3 8 bjork        ;; Euclidean: 3 hits across 8 frame triggers
( "hat" snd . ) 5 8 pbjork       ;; Euclidean: 5 hits across 8 line iterations
```

`every` checks the line iteration count. `bjork` and `pbjork` use Bjorklund's algorithm to distribute k hits as evenly as possible across n positions. `bjork` counts by frame triggers, `pbjork` counts by line iterations.

## Cycling

Cagire has built-in support for cycling through values:

```forth
60 64 67 3 cycle note
```

Each time the frame runs, a different note is selected.

Two cycling words exist:

- `cycle` — selects based on `runs` (how many times this frame has triggered)
- `pcycle` — selects based on `iter` (how many times the line has looped)

When the selected value is a quotation, it gets executed. When it is a plain value, it gets pushed onto the stack.

## Polyphonic Parameters

Parameter words like `note`, `freq`, and `gain` consume the entire stack. If you push multiple values before a param word, you get polyphony:

```forth
60 64 67 note sine snd .    ;; emits 3 voices with notes 60, 64, 67
```

This works for any parameter and for the sound word itself:

```forth
440 880 freq sine tri snd .    ;; 2 voices: sine at 440, tri at 880
```

When parameters have different lengths, shorter lists cycle:

```forth
60 64 67 note           ;; 3 notes
0.5 1.0 gain            ;; 2 gains (cycles: 0.5, 1.0, 0.5)
sine snd .                ;; emits 3 voices
```

## Sova-specific Adaptations

If you've used the original standalone Cagire, a few things are different in Sova:

### Terminology

- "Step" in original Cagire = **Frame** in Sova
- "Pattern" in original Cagire = **Line** in Sova
- "Pattern bank" = **Scene** or line bank

### Scoped Variables

Original Cagire variables are global. In Sova, the default scope is **Instance** (local to the script). Use prefixes to share data:

- `!G.x` / `@G.x` — Global (all scripts in the session)
- `!L.x` / `@L.x` — Line (all frames in the same line)
- `!F.x` / `@F.x` — Frame (persists across runs of the same frame)

See the **Variables** article for details.

### Output Model

Cagire scripts in Sova do not output directly to an audio engine. The `.` word sends events to Sova's scheduler, which forwards them to the world thread for dispatch to MIDI, OSC, and audio devices.

### Tempo and Speed

`tempo!` sets the global tempo of Sova's scheduler (shared across all scripts and lines). `speed!` sets the speed multiplier of the current line.

## Summary

Cagire is a domain-specific language for music. It keeps Forth's elegance (stack, postfix, definitions) but adapts it for live coding.
