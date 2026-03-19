Sova is a music programming environment designed as part of a research/creation project supported by [Athénor CNCM](https://www.athenor.com/) and the [LS2N laboratory](https://www.ls2n.fr/) at the University of Nantes. Sova is developed since 2025 with the goal of providing a flexible, robust and extensible tool for (networked) musical [live coding](https://toplap.org).

Live coding is a technique where a programmer writes code in real-time in front of an audience. It is a way to experiment with code, to share openly, to express oneself through code. It can be technical, poetical, weird, preferably all at once. Live coding can be used to create music, visual art, and other forms of media. Live coding is an autotelic activity: doing it is its own reward. Errors are fun, glitches are cool. Learn more at [TOPLAP](https://toplap.org) or [livecoding.fr](https://livecoding.fr).

## Design

Sova is designed as a vessel for experimentation. It is a runtime made to experiment with programming languages specialized in music performance and improvisation. We want to encourage musicians to develop a performative and expressive approach to computer programming — to think of the computer and its languages as musical instruments. This is both a technical and a poetical endeavour, and we think the two are better tackled together.

Sova can be considered as a step sequencer. In Sova, each step's behavior is defined by code — a script written in a bespoke programming language. Each script can generate an arbitrary amount of musical events and can be as simple or complex as the moment demands: a single note, or some arcane musical process. Unlike conventional step sequencers, the duration of a step is not fixed. Any step can be very short or infinitely long. For each step, one can choose a different programming language — most likely the one that best suits the task at hand. Scripts can be interrupted, modified, and reprogrammed in real-time.

Sova is accessible to beginners while being flexible enough for experienced musicians and artists. No knowledge of Rust is required to use or extend it. New programming languages, I/O devices, and synthesis options can be added without modifying the core. Sova is designed to evolve naturally with its community, both musicians and developers.

## Polyglot approach

Sova is a polyglot programming environment. The environment can host multiple programming languages, both interpreted and compiled. All these languages get access to the same virtual machine, scheduler and I/O provided by the environment. All languages are considered equal, and all the available languages can talk to the same infrastructure, same unified shared memory, etc. Each language is free to follow a specific paradigm, to use peculiar data structures, to expose different abstractions for the musician to play with. Live coders are known to be creative with programming languages. They hack their way in order to get a language that feels nice to play with. Sova aims to foster experimentation around programming languages made for _immediate_ musical expression.

Sova's core and server handle the transformation of all submitted scripts written in high-level languages to an intermediate machine representation close to assembly. Scripts written in very different languages can coexist and be executed concurrently. New languages can be added provided they can be compiled or interpreted into the intermediate representation used by Sova.

## Audience

Sova is designed to support students learning programming and/or computer music. The software is accessible to any musician. No prerequisites are necessary to get started. The most dedicated users will soon feel the call to modify the tool itself as a way to extend their practice. Sova is free and open source: it can be freely modified. software is also designed for experienced musicians, artists and researchers. Sova allows for precise control of musical and audiovisual performances. Sova is all at once: an extensible and open source platform, a collaborative and real-time musical sequencer, an algorithmic and reactive musical instrument.

## Project

Sova is developed by Raphaël Forment, Loïg Jezequel, and Tanguy Dubois. We try to build libre and open source software where all contributions are welcome, in any form! Sova is licensed under AGPL-3.0.
