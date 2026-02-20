# MIDI

Cagire speaks MIDI. You can send notes, control changes, and other messages to external synthesizers, drum machines, and DAWs. You can also read incoming control change values from MIDI controllers and use them to modulate your scripts.

## Device Slots

Sova provides 16 device slots, numbered `1` through `16`. Each slot can connect to a MIDI output or input device. Configure your MIDI devices in the Sova desktop client. Slot `1` is used by default.

## Sending MIDI

The `.` word emits both audio and MIDI messages — the system determines what to send based on the parameters you've set and the device configuration. Build up parameters on the stack, then emit:

```forth
60 note 100 velocity .   ;; MIDI note: middle C, velocity 100
c4 note 80 velocity .    ;; same pitch, lower velocity
```

### Note Messages

| Parameter | Stack | Range | Description |
|-----------|-------|-------|-------------|
| `note` | `(n --)` | 0-127 | MIDI note number |
| `velocity` / `vel` | `(n --)` | 0-127 | Note velocity |
| `chan` | `(n --)` | 1-16 | MIDI channel |
| `dur` | `(f --)` | beats | Note duration |
| `dev` | `(n --)` | 1-16 | Output device slot |

Channel defaults to `1`, device defaults to `1`.

### Control Change

Set both `ccnum` (controller number) and `ccout` (value) to send a CC message:

```forth
74 ccnum 64 ccout .    ;; CC 74, value 64
1 ccnum 127 ccout .    ;; mod wheel full
```

| Parameter | Stack | Range | Description |
|-----------|-------|-------|-------------|
| `ccnum` | `(n --)` | 0-127 | Controller number |
| `ccout` | `(n --)` | 0-127 | Controller value |

### Pitch Bend

Set `bend` to send pitch bend. The range is `-1.0` (full down) to `1.0` (full up), with `0.0` as center:

```forth
0.5 bend .     ;; bend up halfway
-1.0 bend .    ;; full bend down
```

### Channel Pressure

```forth
64 pressure .   ;; medium pressure
```

### Program Change

```forth
0 program .     ;; select program 0
127 program .   ;; select program 127
```

### Message Priority

When multiple message types are set, only one is sent per emit. Priority order:

1. Control Change (if `ccnum` AND `ccout` set)
2. Pitch Bend
3. Channel Pressure
4. Program Change
5. Note (default)

To send multiple message types, use multiple emits:

```forth
74 ccnum 100 ccout .   ;; CC first
60 note 100 velocity . ;; then note
```

### Selecting a Device

Use `dev` to target a specific output slot:

```forth
2 dev 60 note 100 velocity .   ;; send to device slot 2
```

## Reading MIDI Input

Read incoming MIDI control change values with the `ccval` word. This lets you use hardware controllers to modulate parameters in your scripts.

The `ccval` word takes a CC number and channel from the stack, and returns the last received value:

```forth
1 1 ccval   ;; read CC 1 (mod wheel) on channel 1
```

Stack effect: `(cc chan -- val)`

The returned value is `0`-`127`. If no message has been received for that CC/channel combination, the value is `0`.

### Scaling CC Values

CC values are integers `0`-`127`. Normalize to `0.0`-`1.0` first, then use `range` to scale:

```forth
;; normalize to 0.0-1.0
74 1 ccval 127 /

;; scale to custom range (e.g., 200-4000)
74 1 ccval 127 / 200 4000 range

;; bipolar range (-1.0 to 1.0)
74 1 ccval 127 / -1 1 range
```

The `range` word takes a normalized value (`0.0`-`1.0`) and scales it to your target range: `(val min max -- scaled)`.

### Practical Examples

Map a controller knob to filter cutoff:

```forth
74 1 ccval 127 / 200 2740 range lpf
```

Use mod wheel for vibrato depth:

```forth
1 1 ccval 127 / 0 0.5 range vibdepth
```

Crossfade between two sounds:

```forth
1 1 ccval 127 /    ;; normalize to 0.0-1.0
dup saw s swap gain .
1 swap - tri s swap gain .
```

## Real-Time Messages

Transport and clock messages for external synchronization:

| Word | Description |
|------|-------------|
| `mclock` | Send MIDI clock pulse |
| `mstart` | Send MIDI start |
| `mstop` | Send MIDI stop |
| `mcont` | Send MIDI continue |

These ignore all parameters and send immediately. MIDI clock requires 24 pulses per quarter note, so you need to call `mclock` at the appropriate rate for your tempo.
