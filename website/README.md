# Sova: a polyglot live coding environment

_Sova_ is a music software and programming environment designed as part of a research/creation project supported by [Athénor CNCM](https://www.athenor.com/) in Saint-Nazaire and by the [LS2N laboratory](https://www.ls2n.fr/) at the University of Nantes. This software open source licensed and freely available. It is developed by a [small team](docs/team/team.md) of developers and volunteer contributors. Sova is written in [Rust](https://www.rust-lang.org/), and host compilers/interpreters for different programming languages. 

Sova is a multifaceted programming environment built for [live coding](https://toplap.org). On top of all things, it is a tool for artistic experimentation. Sova is a runtime for experimenting with different programming languages specialized in music performance and improvisation. Sova tries to encourage musicians towards a performative and expressive approach to computer programming. It tries to encourage the user  to perceive the computer as a musical instrument: a technical and poetic object.

### Core principles

Sova is based on a familiar concept: the step sequencer. Traditionally, a step sequencer plays one musical event per step at regular intervals. Sova takes a different approach. Here, each step is associated with a _script_ that can generate any number of musical events. Unlike conventional step sequencers, step duration is not fixed — it can be very short or infinitely long. Scripts can be written in various programming languages, each with its own syntax and semantics. The musician can choose the language that best suits their needs for each step. This approach allows for a high degree of flexibility and creativity in musical expression. Scripts can be interrupted, modified, and reprogrammed in real-time, allowing for dynamic and spontaneous musical performances.

![Nested structure of a Sova scene](assets/diagrams/test_diagram.svg)

Sova can sequence events, manipulate data, modify the runtime behavior of the scheduling engine. It is capable of sending and receiving [MIDI](docs/core/devices/midi) and [OSC](docs/core/devices/osc), to pilot its own internal synthesis/sampling engine, etc. Scripts are executed with metronomic precision and tempo/scheduling is steady thanks to a precise networked musical clock ([Ableton Link](https://www.ableton.com/en/link/)). The musician can control every aspect of the performing environment.

## About scripts and programming languages

Sova is a _polyglot_ environment. It can host multiple programming languages -- both interpreted and compiled. Each language can be specialized in describing musical events or sequences in a particular way, or expose different abstractions and paradigms to the user.  Some languages will specialize in writing straightforward melodic/harmonic sequences, others in abstract generative processes, etc. The Sova server handles the transmission of these scripts, written in high-level languages, to an internal machine representation, close to assembly. If the communication protocol with the server is respected, scripts written in very different languages can coexist and be executed without problem on the server. Different languages can be added provided they can be compiled/interpreted into the intermediate representation used by Sova's internal event engine. At the foundation of Sova is a generic and powerful language for abstractly describing musical programs in a synchronous/imperative form.


### Who is it for?

Sova was designed to support learning programming and/or computer music. The software is therefore accessible to any beginner musician. No technical or musical prerequisites are necessary to get started. All complexity arises from the gradual mastery of the tool that the musician acquires through experimentation and play. Using Sova begins with learning the most elementary musical and technical concepts: the music theory specific to _live coding_. Learning then extends toward mastering more advanced programming/composition techniques. The most dedicated users can even modify the tool itself. They will thus possess complete mastery of the instrument and make it evolve with them. The tool is designed to be intuitive. It only gradually exposes the complexity of its operation, always at the musician's initiative.

This software will also interest more experienced musicians and artists. They will find in Sova a tool allowing precise control and synchronization of their various machines, synthesizers, sound/visual generation software. Sova is all at once:
- an extensible, _open source_, multi-language programming and prototyping environment.
- a collaborative (multi-client) and real-time musical sequencer.
- an algorithmic and reactive musical instrument.

Sova can be used to prepare complex musical performances. It can also help the musician formalize while improvising certain playing techniques and/or ways of thinking about musical writing and performance: algorithmic composition, generative stochastic, random, etc.

![First Sova sequence](assets/images/first_line.jpg)
*First musical sequence compiled with Sova (March 2025). Left: raw program, right: emitted messages.*

### What programming languages does Sova support?

Sova is designed to support different programming languages built _ad hoc_ for the software. These languages are specialized in describing musical events or sequences. Each _script_ can be programmed, as needed, using a different programming language. Some languages will naturally specialize in writing melodic-harmonic sequences, others in describing rhythms, events, or more abstract processes. The Sova server handles the transmission of these _scripts_, written in high-level languages, to an internal machine representation, close to assembly. If the communication protocol with the server is respected, scripts written in very different languages can coexist and be executed without problem on the server. Different languages can be added provided they can be compiled/interpreted into the intermediate representation used by Sova's internal event engine. At the foundation of Sova is a generic and powerful language for abstractly describing musical programs in a synchronous/imperative form.

![Client-server architecture](assets/images/test_export.svg)
*Client/server architecture, multiple script languages are interpreted into a single internal representation.*

Being able to build different languages and choose which one to use depending on the situation, device and/or project allows freely exploring different ways of programming and thinking about music. Each programming language also induces a different relationship between the musician and the instrument. Musicians can choose the abstractions most suited to their playing style, their way of working and collaborating (multi-client play). It is not necessary for developers to master the Rust language to propose new languages. The server has an interface allowing submission of a program serialized in JSON format, which will then be translated into machine language and executed by Sova.