# Getting Started with Cagire

Cagire is a stack-based, Forth-like language for live coding music inside Sova. It is a DSL — a Domain Specific Language — specialized in making live music. Values are pushed onto a stack; words consume and produce stack values. It favors brevity and directness: build sounds by stacking parameters, then emit with `.`.

## Why Forth?

Most programming languages rely on a complex syntax of variables, expressions and statements like `x = 3 + 4` or `do_something(()=>bob(4))`. Forth works differently. It has almost no syntax at all. Instead, you push values onto a stack and apply words that transform them:

```forth
3 4 +
```

The program above leaves the number `7` on the stack. There are no variables, no parentheses, no syntax to remember. You just end up with words and numbers separated by spaces. For live coding music, this directness is quite exciting. All you do is think in terms of transformations and add things to the stack: take a note, shift it up, add reverb, play it.

## The Stack

The stack is where values live. When you type a number, it goes on the stack. When you type a word, it usually takes values off and puts new ones back.

```forth
3 ;; stack: 3
4 ;; stack: 3 4
+ ;; stack: 7
```

The stack is last-in, first-out. The most recent value is always on top. This means that it's often better to read Forth programs from right to left, bottom to top.

## Words

Everything in Cagire is either a number or a word. Words are like functions but conceptually simpler. They have no arguments or return values in the traditional sense. They just manipulate the stack directly.

```forth
dup  ;; duplicate the top value
drop ;; discard the top value
swap ;; swap the top two values
```

Words compose naturally on the stack. To double a number:

```forth
3 dup +  ;; 3 3 + = 6
```

You can also create your own words. They will work just like existing words. There are good reasons to create new words:

- To make synth definitions.
- To abstract some piece of code that you use frequently.
- To share data and processes between different frames.

## Values

Four basic types of values can live on the stack:

- **Integers**: `42`, `-7`, `0`
- **Floats**: `0.5`, `3.14`, `-1.0`
- **Strings**: `"kick"`, `"hello"`
- **Quotations**: `( dup + )` (code as data)

Floats can omit the leading zero: `.25` is the same as `0.25`, and `-.5` is `-0.5`.

Any word that is not recognized as a built-in or a user definition becomes a string on the stack. This means `kick snd` and `"kick" snd` are equivalent. You only need quotes when the string contains spaces or conflicts with an existing word name.

Quotations are special. They let you pass code around as a value. This is how conditionals and loops work. Don't worry about them for now — you'll learn how to use them later.

## The Command Register

Traditional Forth programs print text to a terminal. Cagire builds sound commands instead. This happens through an internal accumulator called the command register. The command register has two parts:
- a **sound name** (what instrument to play)
- a list of **parameters** (how to play it)

Three kinds of words interact with it:

```forth
kick sound      ;; sets the sound name
0.5 gain        ;; adds a parameter
.               ;; emits the command and clears the register
```

The word `sound` (or its shorthand `snd`) sets what sound to play. Parameter words like `gain`, `freq`, `decay`, or `verb` add key-value pairs to the register. Nothing happens until you emit with `.` (dot). At that moment, the register is packaged into a command and sent out.

This design lets you build sounds incrementally:

```forth
"sine" sound
c4 note
0.5 gain
0.3 decay
0.4 verb
.
```

Each line adds something to the register. The final `.` triggers the sound. You can also write it all on one line:

```forth
"sine" snd c4 note 0.5 gain 0.3 decay 0.4 verb .
```

The order of parameters does not matter. You can even emit multiple times in a single frame. If you need to discard the register without emitting, use `clear`:

```forth
"kick" snd 0.5 gain clear    ;; nothing plays, register is emptied
"hat" snd .                  ;; only the hat plays
```

This is useful when conditionals might cancel a sound before it emits.

## How Scripts Run in Sova

Sova organizes music into a **Scene** made of parallel **Lines**, each containing a sequence of **Frames**. Each frame holds a script and a duration in beats.

When the sequencer reaches a frame, it runs the associated script. A script can do whatever it is programmed to do: play a note, trigger a sample, apply effects, generate randomness, or all of the above. Scripts can share code and data with each other. Lines play in parallel, frames play in sequence — the sequencer advances to the next frame in a line when the current frame's duration elapses.

Using Cagire doesn't feel like programming at all. It feels more like juggling with words and numbers. The core loop: you hear a line playing, open a frame, change a word, and immediately hear the difference. Forth's brevity helps — swapping `sine` for `saw` or adding `0.3 verb` at the end is a single edit that reshapes the sound.

## Your First Sounds

Play a sample:

```forth
"kick" snd .
```

Play a MIDI note with a sine oscillator:

```forth
c4 note "sine" snd .
```

A sawtooth wave with lowpass filter and reverb:

```forth
"saw" snd c4 note 0.5 gain 800 lpf 0.4 verb .
```

Chain multiple events:

```forth
60 note . 64 note . 67 note .
```

## Next Steps

See the **The Stack** article to understand how data flows through your programs, then explore the other articles for control flow, variables, harmony, and more. The **Language Reference** has complete documentation of all words.
