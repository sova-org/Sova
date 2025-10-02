# Sova Engine

Sova comes with a real-time polyphonic audio engine designed for live coding and performance. It prioritizes zero-allocation audio processing, sample-accurate timing with microsecond precision event scheduling. It is also designed to be modular by design: you can plug sources, effects, and modulations quite easily in order to extend the engine. This engine can be used as a standalone library, through Sova, or even integrated into other applications if you need to use it elsewhere.


## Options 
```
Options:
  -s, --sample-rate <SAMPLE_RATE>
          Audio sample rate in Hz [default: 44100]
  -b, --block-size <BLOCK_SIZE>
          Audio processing block size in samples [default: 512]
      --buffer-size <BUFFER_SIZE>
          Audio buffer size per channel [default: 1024]
      --max-audio-buffers <MAX_AUDIO_BUFFERS>
          Maximum number of audio buffers for sample storage [default: 2048]
  -m, --max-voices <MAX_VOICES>
          Maximum number of simultaneous voices [default: 128]
      --output-device <OUTPUT_DEVICE>
          Specific audio output device name
      --osc-port <OSC_PORT>
          OSC server port [default: 12345]
      --osc-host <OSC_HOST>
          OSC server host address [default: 127.0.0.1]
      --audio-files-location <AUDIO_FILES_LOCATION>
          Directory path for audio sample files [default: ./samples]
      --audio-priority <AUDIO_PRIORITY>
          Audio thread priority (0-99, higher = more priority, 0 = disable, auto-mapped to platform ranges) [default: 80]
      --list-devices
          List available audio output devices and exit
  -h, --help
          Print help
```