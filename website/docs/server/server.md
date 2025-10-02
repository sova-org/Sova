The server is the beating heart of Sova. It manages connections from clients, connects to the local environment through MIDI and OSC, handles synchronizazion and code execution. Every other piece of Sova depends on the server to function, and everything is facultative except the server.

```
Sova acts as the central server for a collaborative live coding environment.

    It manages connections from clients (like sovagui), handles MIDI devices,

synchronizes state, and processes scenes.

Usage: sova_server [OPTIONS]

Options:
  -i, --ip <IP_ADDRESS>
          IP address to bind the server to

          [default: 0.0.0.0]

  -p, --port <PORT>
          Port to bind the server to

          [default: 8080]

      --audio-engine
          Enable internal audio engine (Sova)

  -s, --sample-rate <SAMPLE_RATE>
          Audio engine sample rate

          [default: 44100]

  -b, --block-size <BLOCK_SIZE>
          Audio engine block size

          [default: 512]

  -B, --buffer-size <BUFFER_SIZE>
          Audio engine buffer size

          [default: 1024]

  -m, --max-audio-buffers <MAX_AUDIO_BUFFERS>
          Maximum audio buffers for sample library

          [default: 2048]

  -v, --max-voices <MAX_VOICES>
          Maximum voices for audio engine

          [default: 128]

  -o, --output-device <OUTPUT_DEVICE>
          Audio output device name

      --osc-port <OSC_PORT>
          OSC server port for audio engine

          [default: 12345]

      --osc-host <OSC_HOST>
          OSC server host for audio engine

          [default: 127.0.0.1]

      --timestamp-tolerance-ms <TIMESTAMP_TOLERANCE_MS>
          Timestamp tolerance in milliseconds for audio engine

          [default: 1000]

      --audio-files-location <AUDIO_FILES_LOCATION>
          Location of audio files for sample library

          [default: ./samples]

      --audio-priority <AUDIO_PRIORITY>
          Audio thread priority (0-99, higher = more priority, 0 = disable, auto-mapped to platform ranges)

          [default: 80]

      --list-devices
          List available audio output devices and exit

      --relay <RELAY_ADDRESS:PORT>
          Connect to relay server for remote collaboration

      --instance-name <INSTANCE_NAME>
          Instance name for relay identification

          [default: local]

      --relay-token <TOKEN>
          Authentication token for relay server (optional)

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```