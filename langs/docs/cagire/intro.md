# Getting Started with Cagire

Cagire is a stack-based, Forth-like language for live coding music. Values are pushed onto a stack; words consume and produce stack values. It favors brevity and directness: build sounds by stacking parameters, then emit with `.`.

## Your first sound

Play a sample with `sound` (or `s` for short) and emit with `.`:

```
"kick" s .
```

`sound` names the sample, `.` sends the event. Without `.`, nothing is emitted.

## MIDI notes

Push a note number, set parameters, emit:

```
60 note 100 velocity .
```

You can chain multiple events with separate `.` calls:

```
60 note . 64 note . 67 note .
```

## Note names

Instead of numbers, use note names directly. They push their MIDI value:

```
c4 note .           ;; middle C (60)
fs4 note .          ;; F sharp 4 (66)
bb3 note .          ;; B flat 3 (58)
```

## Sound parameters

Stack parameters before emitting. Order doesn't matter, only the final state:

```
440 freq "sine" sound .
"snare" s 0.8 gain 2000 lpf .
c4 note 100 vel 0.25 dur .
```

## Variables

Three prefixes for variable operations:

```
440 !freq           ;; store 440 in freq (consumes value)
@freq               ;; recall freq onto stack
440 ,freq           ;; store AND keep on stack
```

## Comments

Two styles:

```
( this is an inline comment )
;; this is a line comment
```

## Colon definitions

Define reusable words with `: name ... ;`:

```
: kick "kick" s 0.9 gain . ;
: hi "hh" s 0.6 gain . ;

kick kick hi kick
```

## Quotations

Quotations are anonymous code blocks delimited by `{ }`. They are first-class values that can be passed to words:

```
{ 0.5 distort } sometimes
4 { @i note . } times
```

Inside `times`, `@i` holds the current iteration index.

## Control flow

Conditional execution with `if/else/then`:

```
coin if 60 note . else 72 note . then
```

Short forms with quotations:

```
{ 60 note . } coin ?        ;; execute if true
{ 72 note . } coin !?       ;; execute if false
```

## Simple probability

Named probability words take a quotation:

```
{ 0.5 distort } sometimes   ;; 50%
{ 2 crush } often            ;; 75%
{ 0.8 verb } rarely          ;; 25%
{ "clap" s . } coin ?       ;; 50% (coin returns 0 or 1)
{ 1 delay } 0.3 chance      ;; 30% (0.0-1.0)
```

## Sequencing

Cycle through values across successive runs:

```
60 64 67 3 cycle note .
```

Euclidean rhythms distribute hits across a pattern:

```
{ "kick" s . } 3 8 bjork
```

## Next steps

See the **Language Reference** article for complete documentation of all words, organized by category.
