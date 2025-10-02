# Sova: a polyglot live coding environment

_Sova_ is a music creation software designed as part of a research project supported by [Athénor CNCM](https://www.athenor.com/) in Saint-Nazaire and by the [LS2N laboratory](https://www.ls2n.fr/) at the University of Nantes. This software is freely available and open source licensed. It is developed by a small team of developers and volunteer contributors. Sova is a multifaceted software. It can be described as both a creative programming environment and a musical sequencer. It is a tool for artistic experimentation, designed as a runtime for various musical programming languages specialized in music performance. Sova is made for [live coding](https://livecoding.fr). Our goal is to develop a new tool to encourage musicians to develop a performative and expressive approach to computer programming. Sova tries to encourages the user to perceive the computer as a musical instrument: a technical, creative and poetic object. Sova seeks to offer an immediate, playful, and embodied experience of musical programming.

## How does it work?

Sova is based on the same principle as the familiar step sequencers of drum machines. We have adapted this model not to play single events for each step but rather scripts. They can be of any duration and complexity, and themselves generate a vast amount of events. Each step is a computer programs. Sova is thus capable of emitting notes and messages to other programs, to modify its own state, etc. 

![Nested structure of a Sova scene](assets/images/scene_demo.svg)


The sequencer environment consists of different connections to external software and/or machines. Multiple sequences of _scripts_ can be played together, interrupted and/or reprogrammed on the fly! Scripts are executed rhythmically, with metronomic temporal precision. The musician has complete algorithmic control over the definition of sequences as well as their execution or the behavior of the sequencer. All the scripts forming a playing session are available to all musicians connected to the same session.

## Who is Sova for?

Sova was designed to support learning programming and/or computer music. The software is therefore accessible to any beginner musician. No technical or musical prerequisites are necessary to get started. All complexity arises from the gradual mastery of the tool that the musician acquires through experimentation and play. Using Sova begins with learning the most elementary musical and technical concepts: the music theory specific to _live coding_. Learning then extends toward mastering more advanced programming/composition techniques. The most dedicated users can even modify the tool itself. They will thus possess complete mastery of the instrument and make it evolve with them. The tool is designed to be intuitive. It only gradually exposes the complexity of its operation, always at the musician's initiative.

This software will also interest more experienced musicians and artists. They will find in Sova a tool allowing precise control and synchronization of their various machines, synthesizers, sound/visual generation software. Sova is all at once:
- an extensible, _open source_, multi-language programming and prototyping environment.
- a collaborative (multi-client) and real-time musical sequencer.
- an algorithmic and reactive musical instrument.

Sova can be used to prepare complex musical performances. It can also help the musician formalize while improvising certain playing techniques and/or ways of thinking about musical writing and performance: algorithmic composition, generative stochastic, random, etc.

![First Sova sequence](assets/images/first_line.jpg)
*First musical sequence compiled with Sova (March 2025). Left: raw program, right: emitted messages.*

## How to interact with Sova?

Sova relies on a client/server architecture. The server coordinates the different clients used by musicians. It organizes the rhythmic and synchronous execution of code, connects to external peripherals and software that make up the environment. The server can be run on a dedicated machine or on the computer of one of the musician users. The server is jointly controlled by all connected clients. Each client takes the form of a dedicated graphical interface for musicians. Clients allow manually programming sequences, playing them, modifying them, stopping them, saving them, etc. Clients can be run on the same machine as the server or remotely, on a remote machine capable of connecting through the network. The connection between client and server is made through the TCP protocol. Each communication is serialized/deserialized in JSON format, allowing Sova to be easily extensible and modularized.

![Example Sova client: sovatui](assets/images/bubocore_client_splash.png)
*Example of a Sova client used for testing: sovatui. In the image, view of the server connection page.*

## What programming languages does Sova support?

Sova is designed to support different programming languages built _ad hoc_ for the software. These languages are specialized in describing musical events or sequences. Each _script_ can be programmed, as needed, using a different programming language. Some languages will naturally specialize in writing melodic-harmonic sequences, others in describing rhythms, events, or more abstract processes. The Sova server handles the transmission of these _scripts_, written in high-level languages, to an internal machine representation, close to assembly. If the communication protocol with the server is respected, scripts written in very different languages can coexist and be executed without problem on the server. Different languages can be added provided they can be compiled/interpreted into the intermediate representation used by Sova's internal event engine. At the foundation of Sova is a generic and powerful language for abstractly describing musical programs in a synchronous/imperative form.

![Client-server architecture](assets/images/test_export.svg)
*Client/server architecture, multiple script languages are interpreted into a single internal representation.*

### Code examples

**Example 1a:** a user script (language *BaLi* for _Basic Lisp_).
```lisp
// Sending a note
(@ 0 (n c 90 1))
```

**Example 1b:** the same program in internal notation (_Rust_).
```rust
let note: Program = vec![Instruction::Effect(
    Event::MidiNote(
        60.into(),
        90.into(),
        1.into(),
        TimeSpan::Beats(1.0).into(),
        midi_name.clone().into(),
    ),
    TimeSpan::Micros(1_000_000).into(),
)];
```

Being able to build different languages and choose which one to use depending on the situation, device and/or project allows freely exploring different ways of programming and thinking about music. Each programming language also induces a different relationship between the musician and the instrument. Musicians can choose the abstractions most suited to their playing style, their way of working and collaborating (multi-client play). It is not necessary for developers to master the Rust language to propose new languages. The server has an interface allowing submission of a program serialized in JSON format, which will then be translated into machine language and executed by Sova.

## What role does Sova play in a music creation environment?

Sova is a _middleware_ tool: it does not emit any sound. The software occupies an intermediary and mediating position in a music creation environment. It is designed to be used in conjunction with other music creation software, synthesizers, drum machines, signal processing tools, etc. The tool is entirely oriented toward inter-software communication and synchronization. Sova can send or receive MIDI and OSC messages. It can be synchronized through the Ableton Link protocol but also, if needed, through a MIDI clock. The software can also serve as a central controller and metronome for other software or machines.
