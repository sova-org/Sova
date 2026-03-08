# Devices

You want to hear sound. Every event your code produces goes to a device slot.
The device in that slot sends it out as MIDI, OSC, or audio. No device, no
sound.

## Quick setup

Open the Devices panel. You have three options:

1. Connect a MIDI output (hardware port or virtual).
2. Create an OSC endpoint (IP + port, for SuperCollider, Max, etc.).
3. Use the built-in audio engine (Doux) if the server has audio enabled.

Each connection gets assigned to a slot (1--16). Slot 1 is the default -- if
your code doesn't specify a device, events go there.

## MIDI output

Click "Connect MIDI" in the Devices panel. Available ports on your system
appear in the list. Click one to connect it and assign it to a slot.

To create a virtual MIDI port that other applications can see (useful for
routing into a DAW on the same machine), click "Create virtual MIDI".

In Cagire, send a note to a specific slot:

```forth
2 dev c4 note 100 vel .
```

In Bob:

```
DEV 2
>> [note: 60 vel: 100]
```

## OSC output

Click "Create OSC output" in the Devices panel. Enter a name, target IP, and
port. The endpoint appears in your device list, ready to assign to a slot.

OSC events carry the same parameters as MIDI events. The receiving application
(SuperCollider, Max, Pure Data) interprets them however it wants.

## Device slots

Sova has 16 user slots (1--16) and one fixed slot:

- Slot 0 is the Log device. Always present. Events sent here print to the Log
  panel. Good for debugging.
- Slots 1--16 hold your MIDI ports, OSC endpoints, and the audio engine.

Slot 1 is the default device. Slot assignments persist for the session, so keep
them consistent -- your code refers to slot numbers directly.

A single script can address multiple slots:

```forth
1 dev "kick" snd .       ;; drums on slot 1
2 dev c4 note "saw" snd . ;; synth on slot 2
```

If a slot is empty, events sent there are silently dropped.

## MIDI channels

MIDI channels in Sova are 1--16, matching the standard convention. Default
channel is 1. One MIDI port (one slot) can address all 16 channels:

```forth
1 chan 60 note .    ;; channel 1
10 chan 36 note .   ;; drums on channel 10
```

## MIDI input

MIDI input devices can be connected in the Devices panel, but they don't occupy
slots. They feed incoming data into the system. In Cagire, read a CC value:

```forth
1 1 ccval    ;; CC 1 (mod wheel), channel 1
```

See the **MIDI** article in the Cagire documentation for full details on
sending and receiving MIDI.
