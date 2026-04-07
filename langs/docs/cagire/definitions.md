# Words

## What Is a Word?

In Forth, a *word* is the basic unit of execution: the rough equivalent of a function, command, or identifier in other languages. A Cagire script is, at heart, just a sequence of words separated by whitespace. Numbers, strings, builtins like `dup`, `+`, and `snd`, and anything you define yourself are all words.

Words live in a structure called the *dictionary*. When the interpreter reads a token, it looks it up in the dictionary and runs whatever is bound to that name. Numbers and strings are the one special case: rather than being looked up, they push themselves onto the stack.

What makes this arrangement powerful is its uniformity. Defining a new word adds an entry to the same dictionary the builtins live in, so user code and builtin code are indistinguishable at the call site. Once you have written a word, it behaves exactly like the ones that came with the language. The rest of this article is about how to create such words.

## Creating Words

One of Forth's most powerful features is the ability to define new words. A word definition gives a name to a sequence of operations. Once defined, you can use the new word just like any built-in word.

## The Syntax

Use `:` to start a definition and `;` to end it:

```forth
: double dup + ;
```

This creates a word called `double` that duplicates the top value and adds it to itself. Now you can use it:

```forth
3 double    ;; leaves 6 on the stack
5 double    ;; leaves 10 on the stack
```

The definition is simple: everything between `:` and `;` becomes the body of the word.

## Definitions Are Shared

When you define a word in any frame, it becomes available to all frames across all lines. This is how you share code. Define your synths, rhythms, and utilities once, then use them everywhere.

Frame A:
```forth
: bass "saw" snd 0.8 gain 800 lpf ;
```

Frame B:
```forth
c2 note bass .
```

Frame C:
```forth
g2 note bass .
```

The `bass` word carries the sound design. Each frame just adds a note and plays.

## Redefining Words

You can redefine any word, including built-in ones:

```forth
: dup drop ;
```

Now `dup` does the opposite of what it used to do. This is powerful but dangerous. Redefining core words can break things in subtle ways.

You can even redefine numbers:

```forth
: 2 4 ;
```

Now `2` pushes `4` onto the stack. The number two no longer exists in your session. This is a classic Forth demonstration: nothing is sacred, everything can be redefined.

## Removing Words

`forget` removes a user-defined word from the dictionary:

```forth
: double dup + ;
3 double           ;; 6
"double" forget
3 double           ;; error: unknown word
```

This only affects words you defined with `:` ... `;`. Built-in words cannot be forgotten.

## Practical Uses

**Synth definitions** save you from repeating sound design:

```forth
: pad "sine" snd 0.3 gain 2 attack 0.5 verb ;
```

**Transpositions** and musical helpers:

```forth
: octup 12 + ;
: octdn 12 - ;
```

## Words That Emit

A word can contain `.` to emit sounds directly:

```forth
: kick "kick" snd . ;
: hat "hat" snd 0.4 gain . ;
```

Then a frame becomes trivial:

```forth
kick hat
```

Two sounds, two words, no clutter.

## Stack Effects

When you create a word, think about what it expects on the stack and what it leaves behind. The word `double` expects one number and leaves one number. The word `kick` expects nothing and leaves nothing (it emits a sound as a side effect). Well-designed words have clear stack effects. This makes them easy to combine.
