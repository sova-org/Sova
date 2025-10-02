Each synthesis voice has a set of core parameters that define its basic characteristics and behavior. These parameters can be adjusted to shape the voice. These parameters exist for all synthesis types (oscillators, noise, samples, etc.) and can be modulated over time using various modulation techniques.


| Parameter | Aliases | Range | Default | Modulable | Description |
|-----------|---------|-------|---------|-----------|-------------|
| `amplitude` | `amp` | 0.0 - 1.0 | 0.8 | ✓ | Voice amplitude |
| `pan` | - | -1.0 - 1.0 | 0.0 | ✓ | Stereo position (-1=left, 0=center, 1=right) |
| `attack` | `atk`, `a` | 0.0 - 10.0 s | 0.01 | ✓ | ADSR envelope attack time |
| `decay` | `dec`, `d` | 0.0 - 10.0 s | 0.1 | ✓ | ADSR envelope decay time |
| `sustain` | `sus` | 0.0 - 1.0 | 0.7 | ✓ | ADSR envelope sustain level |
| `release` | `rel`, `r` | 0.0 - 10.0 s | 0.1 | ✓ | ADSR envelope release time |
| `duration` | `dur` | 0.0 - ∞ s | 1.0 | ✓ | Voice duration (0.0=infinite until release) |
| `attack_curve` | `ac` | 0.0 - 1.0 | 0.5 | ✓ | Attack curve shape (0=linear, 1=exponential) |
| `decay_curve` | `dc` | 0.0 - 1.0 | 0.5 | ✓ | Decay curve shape (0=linear, 1=exponential) |
| `release_curve` | `rc` | 0.0 - 1.0 | 0.5 | ✓ | Release curve shape (0=linear, 1=exponential) |
| `track` | `t`, `trk` | 0.0 - 15.0 | 0.0 | ✗ | Audio track assignment for routing |


