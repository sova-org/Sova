# Tunings

A *tuning* is a set of pitch positions inside a repeating period, measured in cents. It describes how a single repeating "octave" is divided, but says nothing yet about which of those positions you actually play. That's the job of a scale, built on top of a tuning.

This separation lets you write microtonal music (19-EDO, just intonation, custom temperaments) using the same scale words as standard 12-tone material.

Cagire ships with one tuning constructor for the common case (`edo`) and one for arbitrary intervals (`tuning`).

## Equal divisions

`edo n` builds an `n`-step equal division of the 2/1 octave (1200 cents):

```forth
12 edo              ;; 12-step tuning over 2/1, standard chromatic
19 edo              ;; 19-EDO tuning, microtonal
24 edo              ;; quarter-tones
```

Stack effect: `(n -- tuning)`. The result is a tuning value with `n` evenly spaced steps from 0 to (but not including) 1200 cents.

`12 edo` is the implicit tuning used by every built-in scale name like `major`, so you only need to call it explicitly when you want a non-12 division.

## Custom tunings

`tuning` builds a tuning from an explicit list of cents offsets within a period. Provide the offsets, then the period:

```forth
[ 90.225 204.090 294.135 408.000 498.045 588.090 702.000
  792.180 906.270 996.090 1110.045 ] 1200 tuning
```

Stack effect: `([c1 c2 ...] period -- tuning)`. The bracketed list pushes the cent offsets and a count; `tuning` pops them along with the period and returns a tuning.

Rules for the offset list:

- Every offset must satisfy `0 < cents < period`. The 0-cent root is implicit and prepended automatically. Don't list it.
- Offsets must be strictly ascending.
- The period itself is in cents and must be positive. Use 1200 for a standard octave; use other values for non-octave periods like Bohlen-Pierce (1902).

The example above is 12-tone Pythagorean tuning over a 1200-cent octave: same number of notes as 12-EDO but with pure perfect fifths (702 cents instead of 700).

## Non-octave example

Bohlen-Pierce divides a 3/1 "tritave" (1902 cents) into 13 equal steps instead of dividing 2/1. You can build it as a tuning and then layer scales on top:

```forth
[ 146.3 292.6 438.9 585.2 731.5 877.8 1024.1 1170.4 1316.7
  1463.0 1609.3 1755.6 ] 1902 tuning
```

Anything that consumes a tuning (like `scale`) treats this exactly like 12-EDO. The period just happens to be 1902 cents instead of 1200, so degrees wrap every 13 steps instead of every 12.
