#show "theTool": "BuboCore"
#show "theLanguage": "BILL"

#set par(justify: true)
#set heading(numbering: "1.1")

#let title = [
  Custom Scripting Languages for theTool
]

#set page(
  footer: align(
    center + horizon,
    title
  ),
)

#align(center, text(17pt)[
  *Messing Around with theLanguage\ or\ #title*
])


theTool has been designed so that it is (relatively) simple for a user to define their own scripting language(s) to be used for Live-Coding the steps of a pattern.
The general idea is to write a compiler that will translate scripts to a low-level language – theLanguage – that is interpreted by the theTool scheduler.
This requires to know theLanguage and to understand how the theTool scheduler works, which is the object of this document.
At the end we also give a few guidelines on how to properly integrate a new script language into theTool.

= The theTool scheduler

== General overview

As show in @fig:overview, the scheduler is responsible for emitting (time-stamped) events. 
These events are mostly sent to the World, the interface between theTool and the different devices — hardware or software — that it controls.
They can also occasionally be sent to other parts of theTool.

For that the scheduler loops forever, executing an "infinite" sequence of steps (each taken into a finite set of steps).
The events that shall be emitted at each of these steps are specified as a sequence of instructions (a program) written in the theTool Intermediate Low-level Language (theLanguage).
So, each step is associated to a theLanguage program.

In order to know how and when each step should occur, theTool scheduler relies on an environment that provides information on everything else (clocks, devices, etc).

#figure(
  image("sched.png", width: 80%),
  caption: [Overview of the theTool scheduler],
) <fig:overview>

== Lifespan of a theLanguage program execution

Each step is always associated to a (potentially empty) theLanguage program.
When the environment is such that a new step shall begin, the scheduler is responsible for instantiating a new execution of the theLanguage program associated to this step (as shown in @fig:steps where programs BP1, BP2 and BP3 respectively correspond to steps 1, 2 and 3).
Once a program execution is instantiated, this program is executed by the scheduler until it is finished (that is, until a normal end of the program is reached, or until an error occurs in the program).

Notice that the duration of a program execution is in general not related to the duration of the step in which it started: it may be shorter or longer.
It is even possible that the same step occurs again before the end of the corresponding program execution, leading to two instances of the same program running at the same time (as for program BP2 in @fig:steps)

#figure(
  image("steps.png", width: 80%),
  caption: [A theLanguage program execution is instantiated at each step],
) <fig:steps>

== How theLanguage programs are executed

A theLanguage program is a sequence of _instructions_ that can either be _control_ instructions (a list of all the control instructions is given in @sec:control) or _effect_ instructions (a list of all the effect instructions is given in @sec:effect).

The effect instructions are the ones that generate emissions of events to the World.
Any effect instruction contains two informations: an event $e$ and a duration $d$.
In the following, such an instruction is denoted by $(e, d)$.

The control instructions are all the other instructions: they are silent from the point of view of the World.
In particular, the kind of instructions that one would expect to find in any assembly language (arithmetic and logic operations, control-flow management) are control instructions.

=== Execution of a single program <sec:execsingle>

In order to execute a program, the scheduler maintains a time counter that states when the next event can be emitted.
This counter is initialized at the current time (so it is possible to emit an event at the very beginning of the program execution).

The scheduler then executes the program instructions one after the other in order.
Depending of the kind of instruction reached by the scheduler, the execution is different:
- a control instruction is executed as soon as it is reached;
- when an effect instruction $(e, d)$ is reached, the scheduler waits until the current time is equal or above the value of its time counter. As soon as this is the case the event $e$ is emitted and the time counter value is set to the sum of the current time and the duration $d$.

This means that control instructions are executed as fast as possible but that the delay between two effect instructions is at least equal to the duration of the first one (notice that this delay could be larger if the duration of the first effect instruction is shorter than the time needed to execute all the control instructions in between the two effect instructions).

=== Execution of several programs in parallel

When several programs execute in parallel (as in step 3 in @fig:steps) each runs as described in @sec:execsingle.
The scheduler executes, in turn, one instruction from each program.
The order in which the programs are considered is the order in which they started their execution.
In case a program shall execute an effect instruction but the time for the event emission has not yet been met, its turn is skipped.

== Variables handling

== Time handling

= theLanguage: theTool Intermediate Low-level Language

== Control instructions <sec:control>

== Effect instructions <sec:effect>

== Read-only global variables

= Guidelines for implementing a new scripting language

== implementing a new scripting langage in RUST

== implementing a new scripting langage using other technologies