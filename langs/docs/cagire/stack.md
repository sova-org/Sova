# The Stack

In most languages, you store values in variables and pass them to functions. Forth has a single shared workspace instead: the stack. You put values on it, words take them off and put results back. Everything flows through the stack. There is no assignment, no named arguments, just a sequence of values waiting to be consumed. Learning to think in terms of the stack is the one skill that makes everything else in Cagire click.

## Pushing Values

When you type a number or a string, it goes on top of the stack:

| Input | Stack (top on right) |
|-------|---------------------|
| `3` | `3` |
| `4` | `3 4` |
| `5` | `3 4 5` |

The stack grows to the right. The rightmost value is the top.

## Words Consume and Produce

Words take values from the top and push results back. The `+` word pops two numbers and pushes their sum:

| Input | Stack |
|-------|-------|
| `3` | `3` |
| `4` | `3 4` |
| `+` | `7` |

This is why Forth uses postfix notation: operands come first, then the operator.

## Stack Notation

Documentation describes what words do using stack effect notation:

```
;; ( before -- after )
```

The word `+` has the effect `( a b -- sum )`. It takes two values and leaves one.
The word `dup` has the effect `( a -- a a )`. It takes one value and leaves two.

## Thinking in Stack

The key to Forth is learning to visualize the stack as you write. Consider this program:

| Input | Stack | What happens |
|-------|-------|--------------|
| `3` | `3` | Push 3 |
| `4` | `3 4` | Push 4 |
| `+` | `7` | Add them |
| `2` | `7 2` | Push 2 |
| `*` | `14` | Multiply |

This computes `(3 + 4) * 2`. The parentheses are implicit in the order of operations.

## Rearranging Values

Sometimes you need values in a different order. Stack manipulation words like `dup`, `swap`, `drop`, and `over` let you shuffle things around. Here is a common pattern: you want to use a value twice.

| Input | Stack |
|-------|-------|
| `3` | `3` |
| `dup` | `3 3` |
| `+` | `6` |

The word `dup` duplicates the top, so `3 dup +` doubles the number.

Another pattern: you have two values but need them swapped.

| Input | Stack |
|-------|-------|
| `3` | `3` |
| `4` | `3 4` |
| `swap` | `4 3` |
| `-` | `1` |

Without `swap`, `3 4 -` would compute `3 - 4 = -1`. With `swap`, you get `4 - 3 = 1`.

## Stack Errors

Two things can go wrong with the stack:

* **Stack underflow** happens when a word needs more values than the stack has. If you write `+` with only one number on the stack, there is nothing to add. The script stops with an error.

```forth
3 +    ;; error: stack underflow
```

The fix is simple: make sure you push enough values before calling a word. Check the stack effect in the reference if you are unsure.

* **Leftover values** are the opposite problem: values remain on the stack after your script finishes. This is less critical but indicates sloppy code. If your script leaves unused values behind, you probably made a mistake somewhere.

```forth
3 4 5 + .    ;; plays a sound, but 3 is still on the stack
```

The `3` was never used. Either it should not be there, or you forgot a word that consumes it.

There are other 'errors' that will not and cannot be caught. It is up to you, most of the time, to make sure that the stack is used correctly.
