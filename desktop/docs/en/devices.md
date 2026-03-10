# Devices

Sova connects to anything that speaks MIDI or OSC: hardware synths, DAWs,
modular software, controllers. The built-in audio engine (Doux) also lets you
produce sound without any external gear. Each event your code produces goes to a
device slot. The device in that slot sends it out as MIDI, OSC, or audio.

## Quick setup

Open the Devices panel. Three options:

1. Connect a MIDI output (hardware port or virtual).
2. Create an OSC endpoint (IP + port, for SuperCollider, Max, etc.).
3. Use the built-in audio engine (Doux) if the server has audio enabled.

Each connection is assigned to a slot (1–16). Slot 1 is the default — if your
code does not specify a device, events go there.

## MIDI output

Click "Connect MIDI" in the Devices panel. Available ports on your system
appear in the list. Click one to connect it and assign it to a slot.

To create a virtual MIDI port visible to other applications (useful for routing
into a DAW on the same machine), click "Create virtual MIDI".

## OSC output

Click "Create OSC output" in the Devices panel. Enter a name, target IP, and
port. The endpoint appears in your device list, ready for slot assignment.

OSC events carry the same parameters as MIDI events. The receiving application
(SuperCollider, Max, Pure Data) interprets them according to its own
conventions.

## Device slots

Sova has 16 user slots (1–16) and one fixed slot:

- Slot 0 is the Log device. Always present. Events sent here appear in the Log
  panel. Useful for debugging.
- Slots 1–16 hold your MIDI ports, OSC endpoints, and the audio engine.

Slot 1 is the default device. Slot assignments persist for the session — keep
them consistent, as your code refers to slot numbers directly. A single script
can address multiple slots by switching device mid-script. If a slot is empty,
events sent there are silently dropped.

## MIDI channels

MIDI channels go from 1 to 16, matching the standard convention. Default
channel is 1. A single MIDI port (one slot) can address all 16 channels,
letting you control multiple instruments from one connection.

## MIDI input

MIDI input devices connect in the Devices panel but do not occupy a slot. They
feed incoming data into the system — CC values, notes, and other messages
become available for your scripts to read. See the language tabs for how to
access incoming MIDI data.
