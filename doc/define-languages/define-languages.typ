#show "theTool": "BuboCore"
#show "theLanguage": "BILL"

#set par(justify: true)
#set heading(numbering: "1.1")
#set raw(lang: "rust")

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

== How variables are handled

theLanguage programs can manipulate variables with control instructions and use them in effect instructions.
These variables are of four kinds: environment variables, global variables, persistent variables and ephemeral variables.

=== Environment variables

From the point of view of theLanguage programs, environment variables are read-only variables.
Their values are set by the environment (think of time informations, random values, etc).
A list of these variables is given in @sec:envvariables.

=== Global variables

Global variables are shared among all the theLanguage program executions.

=== Persistent variables

Persistent variables are local to the theLanguage program in which they are declared but are shared between all the executions of this program.

=== Ephemeral variables

Ephemeral variables are local to the theLanguage program execution in which they are declared.
So, if several executions (parallel or not) of the same program exist, each of these executions has its own version of these variables.

=== Variables with similar names

In the case where several variables with the same name exist, the one with the smallest scope is used.
In other words: 
- if there is an ephemeral variable $v$ declared by some instruction $i$ in a theLanguage program and there exists an environment variable, a global variable, or a persistent variable also called $v$, then any reading or writing to $v$ after the execution of $i$ will be on the ephemeral variable;
- if there is a persistent variable $v$ declared by some instruction $i$ in a theLanguage program and there exists an environment variable or a global variable also called $v$, then any reading or writing to $v$ after the execution of $i$ will be on the persistent variable;
- if there is a global variable $v$ declared by som instruction $i$ in a theLanguage program and there exists an environment variable also called $v$, then any reading or writing to $v$ in any theLanguage program after the execution of $i$ will be on the global variable.

== A few words on functions

#text(blue)[TODO: à écrire]

= theLanguage: theTool Intermediate Low-level Language

In this section we describe all the control instructions (@sec:control) and all the effect instructions (@sec:effect) available in the theLanguage language.
These instructions use variables and durations and we explain how they behave in @sec:variables and @sec:timing respectively.
We also list the environment variables (@sec:envvariables).

== Types of variables <sec:variables>

Each variable (being environment, global, persistent or ephemeral) and constant has a type.

=== Existing types

The possible types are defined in ``` src/lang/variable.rs```:

#raw("pub enum VariableValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Func(Program),
}
")
#text(red)[TODO: je pense que ce serait bien d'uniformiser, genre Int, Float, Bool, Str, Func ou bien Integer, Floating, Boolean, String, Function. J'ai pris la première option, mais ce n'est peut-être pas possible en Rust si les types sont déjà utilisés ?]

Integers, float, bool and str variables are used to store values that can be read or written by the instructions of a program.

Function (Func) variables are programs themselves, they can be executed by calling them with particular call control instruction. #text(red)[TODO: pas encore implanté]

=== Type casting

#text(red)[TODO: est-ce que cet ajustement des types est déjà fait ?]

Instructions arguments are typed: each instruction expects a particular type for each of its input arguments (unless specified otherwise) and has to respect the type of its (potential) output argument when writing to it.

In order to avoid errors, values that have not the expected type will be casted to the correct type, following the rules given in @tab:casting.
In this table, $bot$ denotes a function that does nothing (the program is an empty vector).

#figure(
  caption: "Type casting rules.",
table(
  columns: 6,
  inset: 10pt,
  fill: (x, y) =>
    if x !=0 and x == y { 
      gray 
    } else if x == 0 or y == 0 {
      green.lighten(80%)
    },
  align: horizon,
  table.header(
    [*From\\To*], [*Int*], [*Float*], [*Bool*], [*Str*], [*Func*]
  ),
  [*Int*], [], [Represented\ as float], [$0  arrow #false$\ $!= 0 arrow #true $], [Decimal\ representation], [$bot$],
  [*Float*], [Rounded\ to int], [], [$0  arrow #false$\ $!= 0 arrow #true $], [Decimal\ representation], [$bot$],
  [*Bool*], [$#false arrow 0$\ $#true arrow 1$], [$#false arrow 0.0$\ $#true arrow 1.0$], [], [$#false arrow$ "False"\ $#true arrow$ "True"], [$bot$],
  [*Str*], [Parsed as int\ (0 if error)], [Parsed as float\ (0 if error)], ["" $arrow #false$ \ $!=$"" $arrow #true$], [], [$bot$],
  [*Func*], [$bot arrow 0$\ $!=bot arrow 1$], [$bot arrow 0.0$\ $!=bot arrow 1.0$], [$bot arrow #false$\ $!=bot arrow #true$], [Name of the\ function], [],
)) <tab:casting>


== Control instructions <sec:control>

Control instructions allow to perform basic operations (boolean and arithmetic) over variables.
They also can change the control-flow of a program.

Concretely, a theLanguage program is a vector of instructions. 
At any time, the next instruction to be executed is given by a position in this vector (think of the program counter for a processor) that the scheduler stores.
After executing an instruction, by default this position is increased by one.
To alter the control-flow, a few instructions allow to arbitrarily change this position (jump instructions) or even to change the vector that represents the current program (call and return instructions).

The existing control instructions are defined in ``` scr/lang/control_asm.rs```:

#raw("
pub enum ControlASM {
    // Arithmetic operations
    Add(Variable, Variable, Variable),
    Div(Variable, Variable, Variable),
    Mul(Variable, Variable, Variable),
    Sub(Variable, Variable, Variable),
    // Boolean operations
    And(Variable, Variable, Variable),
    Not(Variable, Variable),
    Or(Variable, Variable, Variable),
    Xor(Variable, Variable, Variable),
    // Bitwise operations
    Bitand(Variable, Variable, Variable),
    Bitnot(Variable, Variable),
    Bitor(Variable, Variable, Variable),
    Bitxor(Variable, Variable, Variable),
    Shiftleft(Variable, Variable, Variable),
    Shiftrighta(Variable, Variable, Variable),
    Shiftrightl(Variable, Variable, Variable),
    // Memory manipulation
    DeclareEphemeral(String, Variable),
    DeclareGlobal(String, Variable),
    DeclarePersistent(String, Variable),
    Mov(Variable, Variable),
    // Jumps
    Jump(usize),
    JumpIf(Variable, usize),
    JumpIfDifferent(Variable, Variable, usize),
    JumpIfEqual(Variable, Variable, usize),
    JumpIfLess(Variable, Variable, usize),
    JumpIfLessOrEqual(Variable, Variable, usize),
    // Calls and returns
    CallFunction(Variable),
    CallProcedure(usize),
    Return,
}
")

=== Arithmetic operations

These instructions are all of the form ``` Op(x, y, z)```.
Arguments x and y are inputs and z is an output.
It is expected that x and y are two numbers of the same type (int or float).
If this is not the case  they will both be casted to float before performing the operation.
The result of the operation will be casted to the type of z (if needed).

Each instruction performs a different operation, as shown in @tab:arithmetic.

#figure(
  caption: "Arithmetic operations semantics",
table(
  columns: 3,
  inset: 10pt,
  align: horizon,
  table.header(
    [*Op*], [*Semantics*], [*Remark*],
  ),
  [Add], [$z <- x + y$], [],
  [Div], [$z <- x \/ y$], [$z <- 0$ if $y = 0$],
  [Mul], [$z <- x times y$], [],
  [Sub], [$z <- x - y$], [],
)) <tab:arithmetic>

=== Boolean operations

These instructions are all of the form ``` Op(x, y, z)``` or ``` Op(x, z)```.
Arguments x and y are inputs and will be casted to bool (if needed).
Argument z is an output.
The result of the operation will be casted to the type of z (if needed).

Each instruction performs a different operation, as shown in @tab:boolean.

#figure(
  caption: "Boolean operations semantics",
table(
  columns: 3,
  inset: 10pt,
  align: horizon,
  table.header(
    [*Op*], [*Semantics*], [*Remark*],
  ),
  [And], [$z <- x and y$], [],
  [Not], [$z <- not x $], [],
  [Or], [$z <- x or y$], [],
  [Xor], [$z <- x xor y$], [],
)) <tab:boolean>

=== Bitwise operations

These instructions are all of the form ``` Op(x, y, z)``` or ``` Op(x, z)```.
Arguments x and y are inputs and will be casted to int (if needed).
Argument z is an output.
The result of the operation will be casted to the type of z (if needed).

Each instruction performs a different operation, as shown in @tab:bitwise.

#figure(
  caption: "Bitwise operations semantics (C-like syntax)",
table(
  columns: 3,
  inset: 10pt,
  align: horizon,
  table.header(
    [*Op*], [*Semantics*], [*Remark*],
  ),
  [Bitand], [$z <- x \& y$], [],
  [Bitnot], [$z <- ~ x $], [],
  [Bitor], [$z <- x | y$], [],
  [Bitxor], [$z <- x \^ y$], [],
  [Shiftleft], [$z <- x << y$], [],
  [Shiftrighta], [$z <- x >> y$], [arithmetic shift],
  [Shiftrightl], [$z <- x >> y$], [logical shift],
)) <tab:bitwise>

== Timing operators <sec:timing>

== Effect instructions <sec:effect>

// arrêter un script, tous les scripts d'un pas, tous les scripts, le dernier script d'un pas, le premier script d'un pas, le dernier script, le premier script

== Environment variables <sec:envvariables>

= Guidelines for building a custom scripting language

In order to build a custom scripting language one needs to be able to compile it to theLanguage.
There are two possibilities for writing a compiler compatible with theTool: 
- compilers written in RUST can be integrated as modules of theTool, and
- compilers built with any technology can be provided as binaries that theTool will call.
The first method is preferred as it allows to directly build theLanguage programs as RUST data-structures and ensures a better integration into theTool.
It also allows to easily distribute new scripting languages: a simple pull request on our Github repository#footnote[https://github.com/Bubobubobubobubo/deep-bubocore] will let us integrate any scripting language into the next versions of theTool. 


== Compiler integration into theTool

Building a custom scripting langage requires to know how to build theTool, please refer to the appropriate document#footnote(text(red)[TODO (quand la doc pour construire theTool sera écrite il faudra la référencer ici)]) for that part.

In order to add a compiler for a new script language one has to comply with the following guidelines:
- create a directory with the name of the language in ``` src/compiler/``` and implement the ``` compiler``` trait by providing a ``` compile``` function that given a script in the language (provided as a string) produces the corresponding theLanguage code,
- create a ```.rs``` file with the name of the language in ``` src/compiler/``` and export (``` pub use```) the ``` compiler``` implementation,
- declare the new module (``` pub mod```) in the ``` src/compiler.rs``` file.

As an example, one can have a look at the _dummylang_ language that has been created for testing purposes while developing theTool:
- the compiler trait is implemented in ``` src/compiler/dummylang/dummycompiler.rs```,
- it is exported in ``` src/compiler/dummylang/dummylang.rs``` by the line ``` pub use dummycompiler::DummyCompiler;```,
- it is declared in ``` src/compiler.rs``` by the line ``` pub mod dummylang;```.

== Compiler as standalone binary

#text(red)[TODO (pas encore possible, à coder d'abord)]