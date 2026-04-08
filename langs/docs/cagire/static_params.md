The simplest way to shape a sound is to hand Doux a number and leave it alone. Set the cutoff to 1200, the gain to 0.7, the attack to 0.01, and let the sound play. Nothing moves over time. This article covers how those numbers flow from the stack to the sound register. The [Control Rate Modulation](#) and [Audio Rate Modulation](#) articles cover what happens when you want the values to change.

If you have not read the [Audio Engine](#) article, start there for the broader picture of what a parameter *is*.

## The Stacking Pattern

Every parameter word pops one value off the stack and attaches it to the current sound:

```forth
saw snd c4 note 0.7 gain 1200 lpf 0.3 verb .
```

Read right to left if the direction is not yet second nature: `0.3 verb` means "take 0.3 off the stack and attach it as the `verb` parameter." Same for `1200 lpf`, `0.7 gain`, and so on. Stack effect: `(v --)`.

The order does not matter. These three lines all produce the same sound:

```forth
saw snd c4 note 0.7 gain 1200 lpf .
```

```forth
saw snd 1200 lpf c4 note 0.7 gain .
```

```forth
saw snd 0.7 gain c4 note 1200 lpf .
```

What matters is that every value you mean as a parameter is on the stack at the moment its parameter word runs. If the stack is empty when a parameter word fires, you get a runtime error.

## Several Values at Once

A few words take several stack values at once. The two common ones are `ad` and `adsr`.

`ad` is a percussive attack and decay shorthand:

```forth
saw snd c3 note 0.01 0.3 ad .
```

`adsr` is the full attack, decay, sustain, release envelope:

```forth
saw snd c3 note 0.01 0.2 0.6 0.4 adsr .
```

Stack effects: `(a d --)` for `ad` and `(a d s r --)` for `adsr`. They are sugar over the individual `attack`, `decay`, `sustain`, `release` words. Writing `0.01 0.3 ad` is identical to writing `0.01 attack 0.3 decay`.

## Overriding

If you set the same parameter twice before `.`, the last write wins:

```forth
saw snd c4 note 0.3 gain 0.8 gain .   ;; plays at 0.8
```

This sounds like a footgun but it is useful. A common idiom is to establish defaults inside a word you define yourself, then override one of them at the call site. Start by defining the voice:

```forth
: bass  saw snd 0.01 0.3 ad 800 lpf 0.7 gain ;
```

Now it plays with its default 800 Hz cutoff:

```forth
c2 note bass .
```

And this overrides the cutoff to 2000 Hz without touching the rest:

```forth
c2 note bass 2000 lpf .
```

See the [Creating Words](#) article for more on defining your own voices.

## Defaults

Any parameter you do not set falls back to a Doux default. For most parameters that means "off" or "neutral": no reverb if you do not set `verb`, no delay if you do not set `delay`, center pan, full gain. Exact default values can change between Doux releases, so treat the word reference as the source of truth and do not rely on implicit values when it really matters.

There is one exception worth naming: when you do not specify `gate`, Cagire fills it in with the current frame duration so the sound lasts exactly one step. Set `gate` yourself to override.

## Chords and Lists as Values

A parameter word does not have to pop a single number. `note` is variadic: push several notes in a row and it eats them all, emitting one voice per note:

```forth
c4 e4 g4 note saw snd 0.4 verb .
```

A named chord works the same way:

```forth
c4 note maj7 chord saw snd 0.4 verb .
```

All voices share the same static parameters. Pitch is the only thing that differs. See the [Chords](#) article for the full chord vocabulary.

## Resetting and Clearing

`reset` removes a single parameter from the register after it was set, reverting it to the Doux default:

```forth
saw snd c3 note 0.5 gain reset gain .   ;; gain is back to default
```

`clear` throws the whole register away without emitting:

```forth
kick snd 0.5 gain      ;; filling the register
clear                  ;; register is emptied
hat snd .              ;; only the hat plays
```

`clear` is the escape hatch when a conditional decides the sound should not play at all. Set up the register unconditionally, then `clear` when you want to cancel.

## Where to Go Next

Static parameters are the foundation. Once you are comfortable with the stacking pattern, there are two ways to make values change over time:

- The [Control Rate Modulation](#) article shows how to vary a parameter from one frame to the next using cycling, randomness, and periodic gating. The value is still a single number by the time Doux hears it. What varies is which number.
- The [Audio Rate Modulation](#) article introduces dedicated words (`lfo`, `slide`, `env`, and family) that hand Doux a continuous movement instead of a single number, so the parameter sweeps smoothly while the sound plays.
