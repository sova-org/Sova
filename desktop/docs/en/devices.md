# Devices

Sova outputs MIDI and OSC. Notes, control changes, and other messages reach external synthesizers, drum machines, DAWs, or any application that listens. The built-in audio engine (Doux) produces sound without external gear. See **Audio Engine** for synthesis details.

## Slots

Devices occupy numbered slots, 1 through 16. Events that do not specify a device go to slot 1 by default. Each language has its own syntax for targeting a slot. See **Events** for what your code can send.

Slot 0 is the Log device. Events sent there appear in the Log panel. Print statements route there automatically.

## Connecting devices

Open the Devices panel to see available connections. System MIDI ports appear in the list — select one and assign it to a slot. Virtual MIDI ports can also be created; they appear as inputs in other applications on the same machine. Virtual ports work on macOS and Linux but not on Windows.

For OSC, create an output by specifying a name, IP address, and port number. Messages are sent as UDP packets. This is how Sova communicates with SuperCollider, Max, Pure Data, and similar environments.

The audio engine (Doux) occupies a slot like any other device. See **Audio Engine** for details.

## MIDI input

MIDI input devices do not occupy a slot. They feed data into the system — incoming CC values are stored and made available to scripts. See the language tabs for how to read them.

## Channels

MIDI channels range from 1 to 16, matching the standard convention. The default is channel 1. A single port addresses all 16 channels, so one connection can drive multiple instruments.

## Latency

Two distinct mechanisms compensate for timing delays.

**Per-device latency** is a user-adjustable offset, 20 ms by default. It shifts message timestamps forward to account for hardware response time. Adjust it per device in the Devices panel.

**Protocol lookahead** is a fixed offset applied by the world thread for dispatch precision. MIDI messages are sent 2 ms early. OSC and audio engine messages are sent 20 ms early. These values are not user-adjustable.

The two offsets combine: per-device latency adjusts for your hardware, protocol lookahead ensures the world thread hands off messages in time for the transport layer to deliver them accurately.
