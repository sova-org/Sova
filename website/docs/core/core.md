# Sova Core 

Sova Core is the innermost layer of the environment. It is a mechanism written to execute musical scripts with great temporal accuracy in a virtual environment. Scripts can be either interpreted or compiled depending on the language used.

<div align="center">

```mermaid
graph TD
    C1[Client 1] --> S[Server]
    C2[Client 2] --> S
    C3[Client 3] --> S
    S --> Core[Core]
    Core --> I[Code execution]
    Core --> M[MIDI/OSC]
    Core --> B[Audio Engine]
    Core --> CLK[Clock]

    style Core fill:#4a9eff
```

</div>