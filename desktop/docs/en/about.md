Sova is a music programming environment designed as part of a research/creation project supported by [Athénor CNCM](https://www.athenor.com/) and the [LS2N laboratory](https://www.ls2n.fr/) at the University of Nantes. Sova is developed since 2025 with the goal of providing a flexible, robust and extensible tool for musical [live coding](https://toplap.org).

## Live coding

Live coding is a technique where a programmer writes code in real-time in front of an audience. It is a way to experiment with code, to share openly, to express yourself through code. It can be technical, poetical, weird, preferably all at once. Live coding can be used to create music, visual art, and other forms of media. Live coding is an autotelic activity: doing it is its own reward. There are no errors, only fun. Learn more at [TOPLAP](https://toplap.org) or [livecoding.fr](https://livecoding.fr).

## Design

Sova is designed as a vessel for experimentation. Sova is a runtime made to experiment with various programming languages specialized in music performance and improvisation. Sova encourages musicians towards a performative and expressive approach to computer programming: the computer as a musical instrument, an object both technical and poetical.

Sova can be considered as a step sequencer where the behavior for each step is defined by computer code. Each step is associated with a script written in some bespoke programming language. Each script can generate an arbitrary amount of musical events. Each script can also be of any arbitrary complexity. It can play a single note or deal with arcane musical processes. Unlike conventional step sequencers, the duration of a step is not fixed. Any step can be very short or infinitely long. For each step, one can choose a different programming language, most likely the one that best suits the task at hand. Scripts can be interrupted, modified, and reprogrammed in real-time.

Sova is a musical instrument accessible to beginners while being flexible enough for experienced musicians and artists. No knowledge of Rust is required to use or extend Sova. The software is designed to be extended by the community. New programming languages, I/O devices or synthesis options can be added without modifying the core itself. Sova is designed to evolve with the musician.

## Polyglot approach

Sova is a polyglot programming environment. The environment can host multiple programming languages, both interpreted and compiled. All these languages get access to the same virtual machine, scheduler and I/O provided by the environment.

Each language is free to follow a specific paradigm, to use peculiar data structures, to expose different abstractions for the musician to play with. Live coders are known to be creative with programming languages. They hack their way in order to get a language that feels nice to play with. Sova fosters experimentation around programming languages for immediate musical expression.

Sova's core and server handle the transformation of all submitted scripts written in high-level languages to an intermediate machine representation close to assembly. Scripts written in very different languages can coexist and be executed concurrently. New languages can be added provided they can be compiled or interpreted into the intermediate representation used by Sova.

## Audience

Sova is designed to support students learning programming and/or computer music. The software is accessible to any musician. No prerequisites are necessary to get started. The most dedicated users will soon feel the call to modify the tool itself as a way to extend their practice. Sova is free and open source: it can be freely modified.

This software is also designed for experienced musicians, artists and researchers. Sova allows for precise control of musical and audiovisual performances. Sova is all at once: an extensible and open source platform, a collaborative and real-time musical sequencer, an algorithmic and reactive musical instrument.

## Project

Sova is developed by Raphaël Forment, Loïg Jezequel, and Tanguy Dubois. The software is licensed under AGPL-3.0.
