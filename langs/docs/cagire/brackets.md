# Brackets

Cagire uses three bracket forms. Each one behaves differently.

## ( ... ) — Quotations

Parentheses create quotations: deferred code. The contents are not executed immediately — they are pushed onto the stack as a single value.

```forth
( dup + )
```

This pushes a block of code. You can store it in a variable, pass it to other words, or execute it later. Quotations are what make Cagire's control flow work.

### Words that consume quotations

Many built-in words expect a quotation on the stack:

| Word | Effect |
|------|--------|
| `?` | Execute if condition is truthy |
| `!?` | Execute if condition is falsy |
| `ifelse` | Choose between two quotations |
| `select` | Pick the nth quotation from a list |
| `apply` | Execute unconditionally |
| `times` | Loop n times |
| `map` | Apply to each stack element |
| `cycle` / `pcycle` | Rotate through quotations |
| `bounce` / `pbounce` | Ping-pong through quotations |
| `choose` | Pick one at random |
| `every` / `every+` | Execute on every nth iteration |
| `except` / `except+` | Skip every nth iteration |
| `chance` / `prob` | Execute with probability |
| `bjork` / `pbjork` | Euclidean rhythm gate |

When a word like `cycle` or `choose` selects a quotation, it executes it. When it selects a plain value, it pushes it.

The cycling and selection words (`cycle`, `pcycle`, `bounce`, `pbounce`, `choose`, `index`) are covered in the **Randomness** article. The periodic words (`every`, `except`, `every+`, `except+`, `bjork`, `pbjork`) are also in **Randomness**. The `map` word is documented in the **Language Reference**.

### Nesting

Quotations nest freely:

```forth
( ( c4 note ) ( e4 note ) coin ifelse ) 4 every
```

The outer quotation runs every 4th iteration. Inside, a coin flip picks the note.

### The mute trick

Wrapping code in a quotation without consuming it is a quick way to disable it:

```forth
( kick snd . )
```

Nothing will execute this quotation — it just sits on the stack and gets discarded. Useful for temporarily silencing a line while editing.

## [ ... ] — Square Brackets

Square brackets execute their contents immediately, then push a count of how many values were produced. The values themselves stay on the stack.

```forth
[ 60 64 67 ]
```

After this runs, the stack holds `60 64 67 3` — three values plus the count `3`. This is useful with words that need to know how many items precede them:

```forth
[ 60 64 67 ] cycle note sine snd .
```

The `cycle` word reads the count to know how many values to rotate through. Without brackets you would write `60 64 67 3 cycle` — the brackets save you from counting manually.

Square brackets work with any word that takes a count:

```forth
[ c4 e4 g4 ] choose note saw snd .    ;; random note from the list
[ 60 64 67 ] note sine snd .           ;; 3-note chord (note consumes all)
```

### Nesting

Square brackets can nest. Each pair produces its own count:

```forth
[ [ 60 64 67 ] cycle [ 0.3 0.5 0.8 ] cycle ] choose
```

### Expressions inside brackets

The contents are compiled and executed normally, so you can use any code inside:

```forth
[ c4 c4 3 + c4 7 + ] note sine snd .    ;; root, minor third, fifth
```

## { ... } — Curly Braces

Curly braces are ignored by the compiler. They do nothing. Use them as a visual aid to group related code:

```forth
{ kick snd } { 0.5 gain } { 0.3 verb } .
```

This compiles to exactly the same thing as:

```forth
kick snd 0.5 gain 0.3 verb .
```

They can help readability in dense one-liners but have no semantic meaning.
