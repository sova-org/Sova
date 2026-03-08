# Recording

Live recording captures audio output into a sample. One word to start, the same word to stop. The result is an ordinary sample you can play, slice, and layer.

## rec

`rec` takes a name from the stack and toggles recording:

```forth
"drums" rec     ;; start recording
```

Play something — a pattern, a live input, anything that makes sound. When you're done:

```forth
"drums" rec     ;; stop recording, register sample
```

The recording is now available as a sample:

```forth
drums snd .
```

## Playback

Recorded samples are ordinary samples. Everything you can do with a loaded sample works:

```forth
drums snd 0.5 speed .            ;; half speed
drums snd 0.25 begin 0.5 end .   ;; slice the middle quarter
drums snd 800 lpf 0.3 verb .     ;; filter and reverb
drums snd -1 speed .             ;; reverse
```

## Overdub

`overdub` (or `dub`) layers new audio on top of an existing recording. It wraps at the buffer boundary, so the loop length stays fixed:

```forth
"drums" overdub    ;; start layering onto drums
```

Play new material over the existing content. Stop with the same call:

```forth
"drums" overdub    ;; stop, register updated sample
```

If the target name doesn't exist yet, overdub falls back to a fresh recording.

## Building Layers

Record a foundation, then overdub to build up:

```forth
;; 1. record a kick pattern
"loop" rec
;; ... play kick pattern ...
"loop" rec

;; 2. overdub hats
"loop" dub
;; ... play hat pattern ...
"loop" dub

;; 3. overdub a melody
"loop" dub
;; ... play melody ...
"loop" dub

;; 4. play the result
loop snd .
```

Each overdub pass adds to what's already there. The buffer wraps, so longer passes layer cyclically over the original length.

## Slicing a Recording

Once you have a recording, carve it up:

```forth
loop snd 0.0 begin 0.25 end .    ;; first quarter
loop snd 0.25 begin 0.5 end .    ;; second quarter
loop snd 0.5 begin 0.75 end .    ;; third quarter
loop snd 0.75 begin 1.0 end .    ;; last quarter
```

Combine with randomness for variation:

```forth
loop snd
0.0 0.25 0.5 0.75 4 choose begin
0.5 speed
.
```

## Orbit Recording

`orec` records a single orbit instead of the master output. Useful for capturing one layer (e.g. drums on orbit 0) without bleeding other orbits in:

```forth
"drums" 0 orec     ;; start recording orbit 0
```

Play patterns routed to orbit 0. When done:

```forth
"drums" 0 orec     ;; stop, register sample
```

`odub` overdubs onto a single orbit:

```forth
"drums" 0 odub     ;; overdub orbit 0 onto "drums"
```

`rec` and `overdub` still record the master output as before.
