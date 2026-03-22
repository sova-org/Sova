#import "jim.typ": jim

#show link: underline

#show "C": smallcaps[C]
#show "Rust": smallcaps[Rust]
#show "WebAssembly": smallcaps[WebAssembly]
#show "WebSocket": smallcaps[WebSocket]
#show "Sova": smallcaps[Sova]
#show "Strudel": smallcaps[Strudel]
#show "SuperDirt": smallcaps[SuperDirt]
#show "Bob": smallcaps[Bob]
#show "Bali": smallcaps[Bali]
#show "Forth": smallcaps[Forth]
#show "Boinx": smallcaps[Boinx]
#show "Cagire": smallcaps[Cagire]
#show "TidalCycles": smallcaps[TidalCycles]
#show "Dough": smallcaps[Dough]
#show "Doux": smallcaps[Doux]
#show "bali": smallcaps[BaLi]
#show "internalLanguage": smallcaps[SAIL]
#show "DMX": smallcaps[DMX]

#show "Midi": smallcaps[MIDI]
#show "OSC": smallcaps[OSC]
#show "SuperCollider": smallcaps[SuperCollider]
#show "Pure Data": smallcaps[Pure Data]

#show: jim.with(
    title: block(width: 80%)[Sova: a libre and open source polyglot and\ collaborative live coding environment\ for pedagogy and research on live coding],
  // title: [Sova (Сова) : un environnement de programmation polyglotte, une machine virtuelle, un serveur et un moteur audio pour le live coding collaboratif],
  authors: (
    (name: "Raphaël Maurice Forment", affiliation: [Artiste-chercheur indépendant,\ \TOPLAP, Cookie Collective]),
    (name: "Tanguy Dubois", affiliation: [Nantes Université,\ École Centrale de Nantes,\ CNRS, LS2N, UMR 6004]),
    (name: "Loïg Jezequel", affiliation: [Nantes Université,\ École Centrale de Nantes,\ CNRS, LS2N, UMR 6004]),
  ),
  résumé:
  [
    Sova (Cова) est un environnement de programmation libre et _open source_ (licence AGPL 3.0) pensé pour la pratique collaborative et polyglotte du _live coding_ musical. Implémenté en _Rust_, Sova se compose d'une machine virtuelle dédiée à la création de langages musicaux événementiels, d'une interface client/serveur, d'un moteur de synthèse sonore et d'échantillonnage, et de plusieurs interfaces utilisateur (GUI et TUI). Sova est également capable de synchronisation entre pairs sur le réseau local via le protocole _Ableton Link_. Quatre langages de programmation musicale conçus à des fins de démonstration illustrent la polyvalence de l'architecture : Bali, Bob, Boinx et Cagire. Sova est le fruit d'un projet de recherche-création initié par l'Athénor CNCM entre Raphaël Forment et le laboratoire LS2N de l'Université de Nantes (2025). Le projet est aujourd'hui au cœur d'une initiative de médiation art-sciences portée par l'Athénor CNCM au sein de plusieurs établissements scolaires de la région Pays de la Loire.
  ],
  abstract: [
        Sova (Cова) is a free and open-source programming environment (AGPL 3.0 licensed @agplv3) designed for collaborative and multi-language live coding. Implemented in Rust and leveraging the Ableton Link synchronization protocol, Sova comprises a virtual machine dedicated to hosting event-based musical programming languages, a client/server communication layer, a dedicated audio engine capable of synthesis and creative sampling, and multiple user-facing graphical interfaces (GUI & TUI). Four musical programming demonstration languages — Bali, Bob, Boinx, and Cagire — each adopting a different programming paradigm, illustrate the versatility of the architecture. Sova is the result of a research-creation project established by Athénor CNCM#footnote[Link to the creation centre website: https://athenor.com. Last consulted: February 20, 2026.] between Raphaël Forment and the LS2N laboratory#footnote[Link to the laboratory website: https://www.ls2n.fr/. Last consulted: February 20, 2026.] at Université de Nantes (2025). The project is currently at the heart of an art-science outreach program run by Athénor CNCM (National Centre for Musical Creation) across several schools in the Pays de la Loire region.
  ]
)

#figure(
  image("sova_screenshot.webp", width: 100%),
  caption: [Screenshot of the Sova multiplayer user interface: `sova-frontend` (February 2026).]
) <fig:sova_screenshot>


= Introduction <sec:introduction>

_Live coding_, the practice of writing and modifying code in real time to generate audiovisual and musical performances, typically with the performer's screen projected for the audience, has emerged as a significant field of inquiry in computer music @collinsLiveCodingLaptop2003 and the arts @jackHydraLiveCoding2019. Although the practice predates its formal naming, the founding of TOPLAP#footnote[TOPLAP (Temporal Organisation for the Promotion of Live Algorithm Programming) was established in Hamburg in 2004. See the organisation's wiki: https://toplap.org/. Last consulted: February 20, 2026.] in 2004 marked a pivotal moment of collective self-organization, followed by the rise of the _algorave_ movement#footnote[The term _algorave_ was coined in 2011 by Nick Collins and Alex McLean. The first self-proclaimed algorave was held in London as a warm-up event for the SuperCollider Symposium in 2012.] @collinsAlgoraveLivePerformance2014 in the early 2010s, which brought live coded music into club, festival settings and into popular culture. On the academic front, the _International Conference on Live Coding_ (ICLC#footnote[See the list of past editions and published proceedings: https://iclc.toplap.org/. Last consulted: February 20, 2026.]), held with regularity since 2015, has consolidated an interdisciplinary research community comprehensively documented by Blackwell et al. @blackwell2022live.

A defining trait of live coding is that practitioners tend to blur the boundaries between composition, performance, and instrument-making @forment2025livecoding @Mori2020. Rooted in free software culture and open to DIY practices, the community has long valued the creation of bespoke systems @blackwell2022live. Code is not merely a means to produce sound but is itself an aesthetic material, projected and performed as an integral part of the creative act. In this context, the choice and design of one's tools and languages becomes a deeply personal matter: each environment shapes how a musician thinks about and expresses music @magnussonAlgorithmsScores2011. Over the past two decades, a rich ecosystem of dedicated environments has matured to support this diversity of approaches — from education-oriented platforms such as Sonic Pi#footnote[See the Sonic Pi project website: https://sonic-pi.net/. Last consulted: February 20, 2026.] to deeply specialized pattern languages such as TidalCycles @mcleantidal and its web-based port Strudel @roos_strudel. Yet, despite this wealth of tools, the ability to rapidly design custom languages — shaped to fit specific creative visions or pedagogical contexts — remains difficult to achieve rapidly with existing software and frameworks.

= The origins of Sova <sec:origins>

Pedagogy has long been a concern of the live coding community. Sonic Pi @aaron2016sonicpi @aaron2016partnerships was designed from the ground up to teach programming in schools through music. ChucK @wang2003chuck @van2024chuck, with its roots in laptop orchestras, has a well-established tradition of pedagogical use in academic settings. Gibber @roberts2016educational and EarSketch @xambo2016challenges have been deployed in contexts ranging from middle-school summer camps to university ensembles, while Estuary @ogborn2017estuary relies on zero-installation browser access to facilitate workshop environments. In each of these cases, however, the students receive a ready-made environment — language, working paradigm, libraries — and the pedagogical effort centres on learning to use it. Comparatively little attention has been given to involving learners in the design of the tools themselves: shaping the language, extending the environment, or adapting its behaviour to their own musical practice. This dimension of the practice — digital lutherie, instrument building — is typically reserved for advanced practitioners. Yet live coding, as a practice rooted in hacker ethics @himanen2001hacker and open-source culture @cox2013speaking @blackwell2022live, carries with it the premise that the performer understands and can modify the software. It follows that learning to live code need not stop at mastering a given tool but can extend to participating in its design — reading, questioning, and contributing to the software that underpins the performance environment.

In winter 2024, a collaboration between Athénor CNCM and the LS2N provided the opportunity to work on live coding with high school students.
The objective was for students — most of whom had little or no prior programming experience — to engage in live coding after only a few sessions.
Rather than introducing a single predetermined language, we sought to involve the students in selecting a language suited to their sensibility.
This, however, would have required presenting a wide range of existing languages within a limited timeframe.
It became apparent, once the students were introduced to the ethics and various philosophies that helped to define live coding, that they could themselves contribute to the design of what a live coding language might be. Our goal thus shifted to creating the necessary framework for students to be able to rapidly create new languages and iterate over their own ideas (@fig:tableau). To our knowledge, no existing live coding tool supported this precise workflow.

#figure(
  image("tableau_bw.jpg", width: 100%),
  caption: [Whiteboard from an early architecture brainstorming session about the design of Sova (winter 2024).],
) <fig:tableau>

We therefore decided to develop a new platform for live coding subject to the following constraints:
- usable for live performance (i.e. with precise timing) and supporting collaborative live coding, as the students would have to perform and play together;
- self-contained and able to run on lower-end computers, shipping everything needed to live code out of the box, including a built-in audio engine (@sec:audio_engine), so as to minimise external dependencies and simplify deployment in any school;
- allowing very fast design and integration of new languages, no more than a few hours for a simple language, since we would need to incorporate the students' ideas from one session to the next, typically within less than one month;
- enabling students to produce music quickly and enjoyably from the outset, while offering enough depth for those who wish to explore further.

Thus began the development of Sova, an ongoing effort whose modular architecture facilitates the integration of live coding languages, user interfaces, and protocols for handling external devices used for sound and visuals making (MIDI / OSC, internal audio engine). In this paper, we present its current state and illustrate what it makes possible for musicians, researchers and high-school students through the example of four demonstration languages (Bali, Bob, Boinx, and Cagire) and two user graphical interfaces dedicated to music-making.

= The modular architecture of Sova <sec:architecture>

The constraints outlined in the previous section — precise timing, collaborative networked performance, rapid language prototyping, and self-contained deployment on lower-end hardware — call for an architecture that cleanly separates the temporal execution engine from the languages it hosts. Sova addresses this through a modular design (@fig:overview) in which each major component of the system (live coding languages, user interfaces, a built-in audio engine, and communication protocols) can be developed, replaced, or extended independently. Sova is built as an ecosystem of software modules coupled to a robust and thorough language design and execution system. In stark contrast with the current popularity of web-based live coding platforms, Sova is distributed as a static independent binary that does not rely on a network connection to run at full capability, suitable for classroom settings and limited connectivity.

Languages made for Sova can either be compiled to bytecode or directly interpreted. They plug into the system through a common interface: they receive timing information and shared state from the scheduler, and produce timestamped events destined for output devices : hardware units, creative software and softsynths. This separation is what enables Sova's polyglot capability: because the execution engine is language-agnostic, adding a new language amounts to implementing a single compiler or interpreter module, without modifying the core infrastructure. Similar experiments at building live coding environments structured around language design have already been carried out by Graham Wakefield and Charlie Roberts, although on a more limited scope #cite(<wakefield2017virtual>).


#figure(
  image("architecture.svg", width: 100%),
  caption: [Software architecture overview. On the left, clients and user interfaces. On the right, output devices.],
) <fig:overview>


At its core, Sova relies on two dedicated threads, with a shared _clock_ synchronized across the network via the Ableton Link protocol @goltz2018ableton that provides a common temporal reference between them: 
- The _scheduler_ runs slightly ahead of real time, stepping through scripts written in any supported language and producing timestamped events. 
- The _world_ receives these events and dispatches them to the appropriate devices (audio engines, MIDI ports, OSC endpoints) at appropriate times. Events can be _immediate_ — meaning that they are dispatched at their exact timestamp — or timed, meaning that they are dispatched a bit ahead of their timestamp so that the target device can handle them at the desired time.

Used as a server, this pipeline from clients inputs through a central scheduled execution to a unique real-time output is what allows multiple users to live code simultaneously using various bespoke languages while the system safely maintains timing guarantees.

The _scheduler_, the _world_ and the _clock_ are the core components that underpin the capabilities offered by Sova. They are not exposed to users and cannot be modified, in direct contrast with every other software component that can be altered or modified freely. In particular, one can: 
- create new event-based live coding languages through the addition of compilers and/or interpreters (@sec:interpreter);
- target any kind of hardware or software device by implementing the appropriate protocols (@sec:protocols);
- plug their own user interface by complying with the client/server and TCP interface defined by Sova (@sec:tcp-interface).

== Crafting live coding languages <sec:interpreter>

Languages can be introduced to Sova in a few different ways: by implementing a Rust `Interpreter` (for an interpreted language) or a `Compiler` (for a language that compiles to bytecode), or more experimentally by wrapping an external binary with the special interpreter/compiler helpers that ship with the project. Regardless of the approach, each language must be registered with the central language registry and may optionally implement a `Language` trait to supply syntax and documentation metadata. The simplest and most common path is to write Rust code implementing one of the two core traits. All default languages are stored in the `langs/` crate on Sova's repository.

The `Interpreter` trait basically requires to implement an ```RUST execute_next``` function that executes a few lines of a program in the new language and returns the effects of these lines on the world: side effects, scheduler actions, etc. The `Compiler` trait requires to implement a ```RUST compile``` function that transforms a program in the new language into a program in our internal low level language called internalLanguage (_Sova ASM Internal Language_). The resulting internalLanguage program is then interpreted by the bundled internalLanguage interpreter, effectively serving as a tiny virtual machine in the style of Java's JVM @venners1998java. internalLanguage is low‑level by design, drawing heavy inspiration from assembly languages#footnote[#link("https://github.com/sova-org/Sova/blob/main/core/src/vm/control_asm.rs") gives a list of the assembly-like instructions available in internalLanguage and #link("https://github.com/sova-org/Sova/blob/main/core/src/vm/event.rs") gives a list of the instructions that have effects on the world.].

Finally, when adding a language, one must implement the `Language` trait, making it possible to define syntax highlighting and documentation which will then be automatically integrated into consuming interfaces. There is no need to edit client code to support new languages. Sova's languages are managed by a `LanguageCenter` structure, containing a `Transcoder` and an `InterpreterDirectory`. Compiled languages must be added beforehand to the `Transcoder`, while interpreted ones need to be added to the `InterpreterDirectory`. The practicality of building live coding languages for Sova using these traits has been demonstrated by creating four languages (@sec:langages). 


== I/O devices and protocol communication <sec:protocols>

Interaction between Sova and hardware or digital peripherals is handled through a `DeviceMap`. 
The goal of the `DeviceMap` is to manage connections to devices, assign slots addressable by musicians, but most importantly: translate internal events into protocol messages. 
Each protocol usable with Sova (MIDI, OSC, etc.) has to be implemented. 
This allows the scheduler to be able to form correct messages to devices using any protocol, as the translation is done based on the targeted device type. 
Generated messages are then self-contained and annotated with a pointer to their targeted device. 
This also allows the world to know how to communicate with these devices without querying the `DeviceMap`. A few standard protocols are currently handled. It is up to the user to add any protocol required for a performance, in principle including DMX, serial connections, custom lighting rigs, etc. Only the most versatile protocols have been implemented for now, in accordance with our constraint of rapid prototyping and deployment in educational contexts (@sec:origins). `DeviceMap` configurations can be saved and restored, allowing to store complex configurations that form a performer's setup and/or favorite configuration. In a remote networked session, each musician can configure its own `DeviceMap`, allowing to either centralize the setup on one computer or let each musician decide how to dispatch device messages locally.

== User interface coupling <sec:tcp-interface>

Sova was conceived from the outset as a library: any Rust project can depend on it and drive the scheduler via inter-thread channels, receiving status notifications in return.  In parallel a lightweight TCP server was implemented, exposing the identical control API over a socket-based protocol and effectively offering the same functionality via remote procedure calls. This dual interface — an in-process crate plus a networked daemon — permits an external process to spawn a Sova instance, issue commands, and poll its state using a simple serialization format.  The server emits structured messages that clients decode locally to update their views, encapsulating timing and event data in an application-level protocol.  This client/server architecture greatly simplifies the construction of user interfaces (see @sec:interfaces) and decouples front-ends from the core execution engine. Optionally, `Sova` is also capable of sending _presence indicators_ (cursor position, compilation visual feedback, chat messages) to remote peers, allowing one musician to form a better mental state of other musicians actions during a remote jam session. 

= Scheduler behavior and details <sec:scheduler>

The scheduler is the central component of Sova, on which all other subsystems depend. Its internal clock runs roughly 30ms ahead of real time so that events are precomputed and timestamped before they need to fire; these messages are then dispatched by the _world_ thread with sub-millisecond precision via a priority queue and active polling. The scheduler organizes execution around objects called _scenes_, _lines_ and _frames_, a sequencer-like grid immediately familiar to electronic-music practitioners, yet equally suited to live coding, where scripts are edited mid-performance, and to offline algorithmic composition, where a scene is authored, stored, and rendered without real-time interaction#footnote[Each client ultimately decides how to render the scene on screen, leading to vastly different interfaces, ranging from standard timeline arrangements to music trackers and/or more creative layouts.].

== Data model: `Frames`, `Lines`, `Scenes` (and scripts)

#figure(
  image("scene.png", width: 100%),
  caption: [Diagram of Sova's scene model with `Frames` and `Lines` hosting scripts written in various languages.],
) <fig:scene>

The scheduler handles what we call a _scene_ (@fig:scene). The scene is split into one or more _lines_, each line being constituted of one or more _frames_.
Each of these frames is associated to a _script_: a program in any live coding language supported by Sova. In order to execute its scene, the scheduler concurrently executes all the lines of this scene. The execution of a _line_ consists in the sequential execution of all its _frames_. Finally, executing a _frame_ consists in starting an execution of its _script_: compilation and run on the virtual machine or interpreter.

== Durations

Each frame carries an explicit duration; a line's total length is the sum of its frames' durations. When and how lines restart is governed by the scene's execution mode, of which three exist: _Free_ — each line loops independently when its own duration elapses; _AtQuantum_ — all lines resynchronize at the next beat-grid quantum boundary provided by Ableton Link; _LongestLine_ — all lines wait for the longest one to complete before restarting together. A line may additionally operate in _trailing_ mode, where a new execution begins while the previous one continues, producing overlapping layers. Frame durations serve a different purpose: each frame occupies a span of time within its line, and the next frame fires once the previous frame's duration has elapsed. A frame's execution is instantaneous — it merely launches the corresponding script, whose actual running time depends entirely on the program it contains.

== Scripts

A script pairs a text source with a designated language and is bound to exactly one frame. When a user submits or updates a script, the change can be applied immediately or deferred to the next beat or quantum boundary. In either case, any running execution of the previous version continues undisturbed; it is only at the next frame trigger that the old executions are discarded and the new code takes effect, ensuring glitch-free transitions during live performance. Once started, a script runs until it terminates; an infinite loop therefore runs indefinitely. All the running scripts are executed concurrently by the scheduler, following a simple round-robin algorithm: the scheduler executes one _computation step_ of each active script execution in turn. Because a frame can be re-triggered before its current execution finishes, multiple executions of the same script may coexist and are all serviced by the round-robin. For a compiled language running on the virtual machine, a computation step consists of executing up to a configurable batch of control instructions (16 by default), yielding immediately upon the first effect instruction (i.e. an instruction that produces an event). For an interpreted language, the interpreter itself decides what constitutes one step through the common `Interpreter` trait. This batching ensures that scripts make meaningful progress per round while guaranteeing that no single execution can live-lock the scheduler.

Each computation step yields one of four outcomes:

- _event emission_: the appropriate protocol builds a timestamped message and forwards it to the world;
- _silent progress_: the script advanced its internal state without producing an observable effect;
- _idle request_: the script suspends for a specified duration, during which all other scripts continue to run;
- _termination_: the script has reached its end and is removed from the execution pool.

= Variables and evaluation context <sec:machine_virtuelle>

Programs run by Sova share access to internal data structures that _support inter-program communication_, execution context management, and musical event emission. Sova internally uses typed variables — both primitive and composite — so that new languages can be developed without exposing classical pointers or dynamic allocation. The supported types are: integers, floating-point numbers, high-precision decimals, booleans, strings, durations (representable as microseconds, beats, or fractions of the current frame length), dynamic vectors, dynamic maps with string keys, first-class functions, binary blobs, and generators (values that evolve over time according to a configurable waveform). Conversions and operators are defined for each type. Variables are partitioned into four scopes: _global_ (shared across all scripts), _line_, _frame_, and _instance_ (local to a single script execution).


== Evaluation context

Each script execution receives an evaluation context that allows it to interact with its environment. This context exposes:
- the current logic-time;
- variables at all four scopes: global, line, frame and instance as well as environment values;
- a double-ended stack local to the execution;
- the shared clock for conversions between time units;
- the indices of the current line and frame, together with their iteration counts;
- the length of the frame in beats;
- the structure of the scene and the device map.

Errors raised during execution are collected rather than causing a crash, providing the user with a diagnostic that can be displayed accordingly in software clients.



== Events

In order to achieve maximal modularity and be as protocol-agnostic as possible, events that can be triggered from scripts are either specific to protocols (in order to perform complex actions) or generic. 
Most things can be done with generic events that are then translated internally into protocol messages by the DeviceMap. The same event type could then be used for internal engines, MIDI or OSC devices for instance, without the script needing to know which protocol will be used.

= Audio engine <sec:audio_engine>

#figure(
  image("doux.png"),
  caption: [Screen capture of the audio engine demonstration website (WebAssembly version), available at: https://doux.livecoding.fr. Last consulted: February 27, 2026.]
)

Sova was initially developed with no perceived need for built-in synthesis, signal processing, or sampling capabilities. The rationale was that it could be done through the use of external software such as SuperCollider or Pure Data. Over time, however, the imperative to produce a self-contained and portable piece of software that could be readily deployed across a diverse fleet of machines gradually spurred the design of a lightweight, robust, and autonomous audio engine. To that end, Sova drew on the preliminary research carried out by Felix Roos toward the design of a new audio engine for Strudel @roos_strudel. Over the course of 2025, contributors to the Strudel project began crafting a minimal, self-contained synthesis engine written in the C programming language called Dough#footnote[See the project page on Codeberg: https://codeberg.org/uzu/dough or the demo website: https://dough.strudel.cc. Last seen: February 21, 2026.], capable of running both natively and in the browser as a WebAssembly module. This engine was subsequently ported to Rust, then extended and tailored to meet the specific constraints of the Sova project. The result of this initiative is available to download as Doux#footnote[See the dedicated website: https://doux.livecoding.fr and the GitHub repository: https://github.com/sova-org/doux. Last consulted: February 21, 2026.], drawn as a dependency by the Sova project. Doux has been gradually extended to offer more sound design capabilities for users running it natively.

== Architecture

By embedding synthesis, sampling and DSP effects directly into a single binary/library, Doux addresses the self-containment constraint from @sec:origins, eliminating the dependency on an external sound server#footnote[One can still use SuperDirt, the classic live coding audio engine hosted by SuperCollider through a dedicated pre-configured route kept for convenience]. Unlike dynamically assembled node-graph engines such as SuperCollider @mccartney2002rethinking, Doux employs a fixed-topology signal chain: every voice traverses an identical processing pipeline, and unused stages are bypassed rather than removed from a graph, eliminating graph traversal and dynamic dispatch from the audio thread. The engine pre-allocates a pool of voices in a compact array with $O(1)$ swap-remove deallocation. Each voice is a complete mono/stereo signal chain that sequentially couples oscillators, state-variable and ladder filters with independent envelopes, distortion units, per-voice effects, amplitude envelope (DAHDSR), stereo panning. These voices are then routed to one of eight (by default) stereo _orbit_ buses carrying effects shared by voices such as delay, reverb, comb filter and feedback units with silence-gating. The audio path thus processes sample blocks without heap allocations, in an efficient manner. Any synth voice configuration can be speedily described using an OSC-like message scheme:
#figure(`/sound/saw/note/48~60:0.5/delay/0.5
/delaytime/0.1/delayfb/0.8/coarse/12
/gain/1/width/2/release/8`, caption:[Example of a command message sent to the Doux engine for configuring a fixed-duration synthesis voice featuring delay, sample depth reduction, stereo width panning and pitch audio-rate modulation. ])

== Sound generation and modulation

Sound sources include band-limited oscillators (PolyBLEP) with a phase-shaping pipeline, additive and 3-OP frequency modulation synthesis, white/pink/brown noise, seven synthesized drum types, lazy-loaded sample playback via a lock-free registry. Experimental support for soundfonts (via `.sf2` files) is also currently being tested, allowing users to use soundfonts as raw materials to transform. Live input (microphones, audio internal routing) and live samping with optional overdub is also supported, allowing musicians to generate, use and overwrite audio samples live. We do believe that the basic timbral palette offered by the engine suffices for musically complete performances without any external sample library and without relying on more complex graph-based audio engines. Each voice carries its own voice level effect chain before being summed into its assigned _orbit_ audio bus. Orbit audio busses each apply time sensitive effects such as a delay (standard, ping-pong, tape, or multitap), a reverb (switchable between a Dattorro plate algorithm and a feedback delay network), a comb filter, and a modulated feedback delay. Per-voice effects shape individual timbres; orbit effects provide shared spatial processing across all voices routed to the same bus. Any numerical parameter across both layers can additionally be modulated at audio rate through an inline mini-language embedded directly in parameter values (e.g. `200~4000:2` oscillates between two bounds over a given period), collapsing the traditional modulation matrix into the parameter specification itself.

Doux is a standalone library. The bridge crate `doux-sova` converts Sova's key–value event payloads into Doux command strings, mapping microsecond timetags to engine-local time via the shared clock. Within Sova, Doux registers as a `ProtocolDevice`, sharing the same dispatch path as MIDI and OSC — adding the engine required no modification to the scheduler or world. Doux has already been used pedagogically in at least one other software (Cagire#footnote[See the dedicated website about this project, derived from implementation work done on Sova: https://cagire.raphaelforment.fr. Last consulted: March 15, 2026.]) for an intensive one-week long live coding workshop at the Ecole Nationale Supérieure des Arts Décoratifs (Paris, PSL) in March 2026. It also comes with a standalone REPL, used for testing, and with a non realtime CLI usable to generate sound samples as `.wav` files. 

#pagebreak()

= Interfaces <sec:interfaces>


The separation between Sova's core and its user-facing layer means that interfaces are not privileged components: they are ordinary clients that either embed the core library directly or connect through the server's WebSocket API, enabling both solo and networked operation#footnote[Client/server architectures are well established in the live coding field. SuperCollider's separation of language and synthesis server @mccartney2002rethinking remains unrivalled in flexibility and is the most influential example of this pattern.]. Designing these interfaces, however, presents a particular challenge. Experienced live coders expect rich, feedback-driven editors that tightly couple code representation with musical structure @roberts2015beyond @mclean2010visualisation; because the performer's screen is typically projected for the audience, the interface is itself part of the performance, and its visual identity (fonts, color schemes, layout) must be personalizable to reflect each artist's stage presence. High school students encountering programming for the first time, by contrast, need an environment that is immediately approachable and with sensible defaults. 

In both cases, users must navigate a system whose components (scheduler, audio engine, devices, languages) are deeply interlinked. The interface must expose this complexity without overwhelming or hindering musical action. Beginners should be able to produce music quickly while receiving useful and immediate feedback to guide them while advanced users retain full control and customization capabilities. In practice, this calls for domain-specific code editors and domain-specific clients that borrow familiar affordances from general-purpose programming IDEs (syntax highlighting, code completion, search) while specializing them for the live coding context. Real-time transport controls, scene visualization, audio monitoring and tight integration with the scheduler and audio engine replace the compilation pipelines and debugger panels of conventional IDEs.

Three interfaces, two client code editors and one CLI server utility have been developed to date in order to make Sova usable by musicians. Each tool is conceived to address a different level of access and level of detail and user feedback in the Sova ecosystem:

- (1) `sova-server`: a command line interface for managing Sova server instances using a rich flag configuration system. Used mainly as tooling for testing and/or deploying jam sessions on local networks.
- (2) `solo-tui`: a terminal user interface (TUI) dedicated to running as close as possible to Sova's internals without any network stack.
- (3) `sova-frontend`: a general purpose client / server binary that contains all of Sova's component and can be used as a graphical client and/or server interface. It is aimed to be used by students and musicians alike.

#figure(
  image("sova_full.png", width: 100%),
  caption: [Live session using a redesigned `sova-frontend` GUI (March 14th). On screen: the scene view, code editors, visuals editor and various user-controlled widgets (oscilloscope, VUMeter, etc).],
) <fig:sova_full>

== Command-line server interface

The `sova-server` binary is a headless server accessible through the command line. It exposes a rich flag-based configuration interface covering network, timing, and audio engine / resources settings. This client is dedicated to workshop organizers, so that they can prepare shell scripts encoding exact server configurations, ensuring consistent setup across sessions and machines. A single invocation command can fully capture a particular running configuration. The server exposes the full Sova API over WebSocket: any number of clients can connect simultaneously, each sending editing commands and receiving real-time state updates. In a classroom or workshop setting, this client can be used to deploy a single server on a central machine handling audio synthesis and device routing, while students connect from lightweight clients that need not ship the audio engine or manage hardware. Only the graphical interface and a network connection are required to participate in a collaborative session. The server is also available as a Rust library crate#footnote[In the Rust ecosystem, a _crate_ is a compilation unit that can be shared and reused as a package. See https://doc.rust-lang.org/book/ch07-01-packages-and-crates.html. Last consulted: February 23, 2026.], allowing third-party applications to embed a Sova instance programmatically.

== Solo Terminal User Interface (TUI)

For situations where a graphical desktop is unavailable or unnecessary, Sova provides a standalone terminal user interface built with the Ratatui framework#footnote[Ratatui is a Rust library for building terminal user interfaces. See https://ratatui.rs/. Last consulted: February 23, 2026.] (@fig:tui_screenshot). The TUI bypasses the server entirely: it embeds both the core library and the audio engine directly, driving the scheduler and audio output within a single process without any networking layer. This makes it the lightest deployment option, a single binary that requires no server setup, no WebSocket connection, and no network configuration. `solo-tui` is particularly suited to lower-end machines or experienced users who prefer a minimal environment for solo practice and experimentation. The interface is organized into navigable pages: a scene view renders lines and frames on a canvas, highlighting active playback positions in real time. A code editor provides line numbers, undo/redo, clipboard integration, and per-frame language selection. A configuration page exposes scene save/load, execution mode switching, and per-line looping and trailing controls. Additional pages display connected devices, logs, and shared variables. All interaction is keyboard-driven, with transport controls accessible from any page and a notification system providing immediate feedback on user actions.

#figure(
  image("tui_screenshot.png", width: 100%),
  caption: [`solo-tui` main view displaying the current `Scene` using a grid-like organizational pattern.],
) <fig:tui_screenshot>

== Graphical User Interface

The primary interface for both performance and pedagogy is `sova-frontend`, a desktop application built with the egui immediate-mode GUI library#footnote[Egui is a Rust library for building portable, immediate-mode graphical interfaces. See https://www.egui.rs/. Last consulted: February 23, 2026.] (@fig:sova_screenshot). It connects to an instance of `sova-server` over WebSocket or can start an embedded server from within the application itself. The layout is organized around a central scene grid displaying lines and frames with real-time playback progress, surrounded by togglable panels that can be shown, hidden, or detached into separate windows depending on the performer's needs. A transport bar provides playback controls, tempo display, phase visualization, and execution-mode selection. The built-in code editor supports syntax highlighting for all demonstration languages, with multiple color themes, search, line numbers, and word wrapping. For audio feedback, an oscilloscope, a VU-meter and a 128-band FFT spectrum analyzer run alongside the scene view (@fig:widgets). A sample browser lets users navigate and preview the audio files available to the Doux engine. A log panel with severity filtering displays server and client messages, and a searchable command palette offers quick access to all available actions.

A special visual panel provides an early port of Hydra @jackHydraLiveCoding2019 displayed over `egui`, currently being extended and adapted to enrich the performance visually during public performance. Students involved in the pedagogical workshops described in @sec:origins have already been introduced to it and are familiar with its usage, allowing them to jam both musically and visually.

#figure(
  image("widgets.png", width: 100%),
  caption: [A selection of graphical widgets dedicated to audio monitoring taken from the Sova user interface: an oscilloscope, a spectrum visualiser and a sample bank browser.],
) <fig:widgets>


The collaborative dimension of the graphical interface draws on the experience gained by one of the authors as a contributor to Flok#footnote[See the Flok project website: https://flok.cc. Last consulted: February 22, 2026.], a popular web-based collaborative editor for live coding developed by Damián Silvani @vasilakos2021exploring. In `sova-frontend`, connected peers can see each other and interact with each other through various actions. Each peer's cursor position and current editing activity is indicated within the scene grid, and a built-in text chat allows communication during performance. An in-app documentation panel provides getting-started guides and per-language references. Both the interface and the documentation are internationalized — currently in English and French — so that users from different countries can adopt the tool without a language barrier.


Despite this range of interfaces, several limitations remain due to the project's early stage of development. The code editor, while functional, lacks features that experienced programmers have come to expect from mature environments — notably code completion, inline error reporting, and language-aware navigation. More broadly, all three interfaces remain code-centric: interaction with external controllers — MIDI surfaces, gamepads, or gestural devices — is limited to what the server exposes, with no dedicated mapping or binding per interface/client. A web-based client, which would eliminate installation entirely and open the door to mobile devices and browser-only workflows, has not yet been developed. These are active areas of future work. Feedback from experienced computer musicians would prove beneficial for the future stages of development.


= Languages <sec:langages>


Four demonstration languages, in various stages of completeness,  have been developed for Sova. Each language is adopting a distinct programming paradigm: imperative (Bob), declarative (Bali), pattern-based (Boinx), and concatenative (Cagire). Together they serve as a proof of concept that the architecture described in @sec:interpreter is genuinely language-agnostic. Because all four share the same variable stores, clock, and device map, they interoperate freely within a single scene: a global variable set by a Bob script can be read by a Cagire script running on another line. Two of the languages (Bob and Bali) are compiled to bytecode for the shared virtual machine; the other two (Boinx and Cagire) provide their own interpreters and produce events directly through the common trait interface.

== Compiled languages: Bob and Bali

Bob is an imperative, expression-oriented language that uses Polish (prefix) notation with fixed-arity operators, eliminating the need for parentheses. Inspired by the Monome Teletype hardware sequencer#footnote[See the Monome Teletype documentation: https://monome.org/docs/teletype/. Last consulted: February 23, 2026.] design choices, it is designed for brevity and simplicity, reminiscent of early BASIC implementations with its brutalist upper case. Events are described as key-value maps and emitted with `>>`, while `WAIT` advances a virtual clock through the script. Bob provides four variable scopes (global, line, frame, and instance), a complete set of arithmetic and logical operators, and control structures including conditionals, counted loops, while loops, and a switch statement.

#figure(
```
EU 3 8 0.125:
  >> [note: ADD G.root MUL I 7
      vel: RRAND 60 127]
END
```,
caption: [Bob syntax exemplified through the composition of an Euclidean rhythm applied to a MIDI sequence.]
)

Beyond basic control flow, Bob offers built-in euclidean and binary rhythm generators, probabilistic execution (`PROB`), concurrent branching (`FORK`), lambdas, and higher-order list operations (`MAP`, `FILTER`, `REDUCE`). Source code is parsed by a LALRPOP grammar into an abstract syntax tree, then compiled to bytecode for the Sova virtual machine.

Bali (_Basically a Lisp_) is a declarative language whose (fake) S-expression syntax places timing at the core of the notation. A Bali program is a collection of timed musical effects — notes, control changes, OSC messages — distributed over fractional beat positions within a frame. Structural constructs such as `(loop N ...)`, `(eucloop ...)`, `(spread ...)`, and `(binloop ...)` subdivide the frame's duration into equal fractions, producing rhythmic patterns through nesting rather than explicit wait statements.

#figure(
```
(loop 4
  (note c3 v:90 ch:1)
  (note {e3 g3} v:70))
```,
caption: [Short demonstration of Lisp-like Bali syntax.]
)

A context system, `(with dev:1 ch:2 v:80 ...)`, propagates default values for device, channel, velocity, and duration to all nested effects, reducing redundancy. Non-determinism is available through choice brackets `{...}` (random selection), alternation brackets `<...>` (sequential cycling), and sequence brackets `[...]` inspired by _Uzulang_ languages#footnote[To see a list of existing uzulangs, consult: https://uzu.lurk.org/t/uzulangs/5660. Last consulted: February 27, 2026.]. The compiler, also built on a LALRPOP grammar, expands the AST into a time-sorted list of events and emits Sova virtual machine bytecode. Bali has historically been the first language to be developed for Sova but is currently relatively unused and unmaintained.

== Interpreted languages: Boinx and Cagire

Boinx is a declarative, functional and pattern-based language built in order to visually build complex patterns scheduling events over a time-span. 
A program is constituted of assignments and outputs (which are all executed simultaneously when the script starts). 
The main idea of Boinx is to compose patterns into other patterns using slots (for instance placeholders '`.`' or durations) and collections (simultaneous '`(...)`' and sequentials '`[...]`'). 
To make complex compositions, Boinx uses 5 compositional operators (`|`, `°`, `!`, `~` and `#`).

#figure(
```
min = (. .+3 .+7)
maj = (. .+4 .+7)

[C3 A3 E3 G3] ! [maj min min maj]
~ [. [..] _ .]
# <s: 'saw' note: .>
```,
caption: [Short demonstration of Boinx syntax.]
)

Musical theory is handled in Boinx by the use of macros (`_scalemaj`, `_maj`, `_arpmaj`...) and it has generative aspects through functions (`choice`, `maybe`, `range`...). Subprograms can be started to emulate parallelism using `{ ... }`. 
Lastly, devices and channels can be specified using `@ device : channel` where device and channels can be Boinx objects as well (and thus, patterns). 
Boinx is internally compiled in its own syntax tree, but hosts its variables in the VM, and can therefore interact with other languages.

Cagire is a concatenative, stack-based language inspired by classic Forth @rather1996evolution implementations. It features a very lightweight and approachable syntax: programs are sequences of words and numbers separated by spaces, evaluated in postfix order. Values are pushed onto a shared stack; words consume values from the top and push results back. A central design device is the _command register_, an accumulator that builds sound events incrementally: a sound name is set (`kick snd`), parameters are added (`0.5 gain`, `c4 note`, `0.3 verb`), and nothing is sent until the emit word `.` fires the accumulated command.

#figure(
```
[ saw tri ] choose sound
[ c4 e4 g4 ] cycle note
0.5 gain 5000 100 0.5 slide lpf
2 4 rand fm 0.5 fmh .
```,
caption: [Short demonstration of the Cagire syntax.]
)

Cagire provides a rich vocabulary of stack manipulation words, user-defined words (`: name ... ;`), first-class quotations (`{ ... }`) for deferred code, and an extensive built-in music theory library covering note literals, intervals, chord and scale types, and frequency conversion. Probability words (`coin`, `sometimes`, `rarely`), Euclidean sequencing (`bjork`), generators, and audio-rate modulation words (`lfo`, `slide`, `env`) round out the language. Internally, Cagire compiles source code to its own dedicated opcode set — distinct from the shared Sova virtual machine — but presents itself to the scheduler through the common interpreter trait, producing events directly. Cagire is a direct adaptation of the language used by the eponymous software currently developed by Raphaël Forment#footnote[Project website: https://cagire.raphaelforment.fr. Last consulted: February 27, 2026.].

The diversity of programming paradigms currently supported is deliberate. Each language was developed in a matter of days to weeks, validating the claim that Sova's architecture supports rapid language prototyping. The four paradigms — imperative, declarative, pattern-based, and concatenative — cover a broad region of the design space for event-based musical languages, while sharing the same runtime infrastructure. Musicians can choose the paradigm that best fits their way of thinking about music, or design entirely new ones. Developing these languages as test implementations during our initial work on Sova served as a de-facto validation of the general soundness of the software architecture. 

= Conclusion

In this paper, we presented Sova, a free and open-source modular environment for collaborative live coding built around a central scheduler, a dedicated virtual machine, and a lightweight audio engine. Born out of a concrete pedagogical need, enabling high school students to engage with live coding through languages they could help design, Sova serves a dual purpose: facilitating the rapid development of new event-based musical languages, and providing a solid foundation for teaching and performing live coding. The diversity of paradigms currently supported validates that the architecture is genuinely language-agnostic and that new languages can be prototyped in a matter of days.

Sova is already being put to use in the field. Throughout winter and spring 2026, the core team has deployed Sova in an ongoing pedagogical project initiated by Athénor CNCM, teaching live coding practices to high school students in Nantes, Saint-Nazaire, and Guérande (@fig:students). Early feedback from these sessions is being actively collected and will inform subsequent iterations of both the software and its languages. In parallel, Sova has been introduced to the international live coding community through its presentation at Equinox, an independent conference hosted by TOPLAP Italia#footnote[See the Equinox conference website: https://equinoxtoplap.it/. Last consulted: March 15, 2026.]. We are actively seeking feedback from experienced researchers and computer musicians who may wish to use Sova for their creative or academic projects, and welcome any contribution or initiative that could help take the project further.

#figure(
  image("students.jpeg", width: 100%),
  caption: [High school students engaged in a live coding session using Sova during a workshop organized by Athénor CNCM.],
) <fig:students>

Several challenges remain. The software has not yet reached full stability across all supported platforms, and certain bugs — particularly those related to the audio engine and its behaviour on lower-specification hardware — can occasionally interrupt students' workflow during workshop sessions. On the language front, continued effort is needed to lower the barrier to language integration, to mature the existing demonstration languages, and to provide richer examples and internal documentation throughout the codebase. While this work is underway, the onboarding experience for newcomers — students in particular — could be made considerably smoother. Addressing these reliability and usability concerns is a priority for the near-term development effort. Sova's source code is released under the AGPL 3.0 license and is available on GitHub#footnote[https://github.com/sova-org/Sova], alongside a companion documentation and research website#footnote[https://sova.livecoding.fr].



= Acknowledgments

The authors wish to thank Athénor#footnote[https://www.athenor.com/], Centre National de Création Musicale, for initiating and supporting the pedagogical workshops that have been central to Sova's development and early validation. Raphaël Forment would like to warmly thank Tanguy Dubois and Loïg Jezequel for their invaluable contributions to the design and formal specification of Sova's programming languages, and more broadly for their sustained engagement with the project throughout its development.

#bibliography("references.bib", title: smallcaps[References], style: "ieee")
