#show "theTool": "BuboCore"
#show "theLanguage": "BILL"

#set par(justify: true)
#set heading(numbering: "1.1")
#set raw(lang: "rust")

#let smallCell(body, factor: 7pt) = {
  text(factor)[#body]
}

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

#text(red)[TODO: pattern = tableau de sequences, sequence = tableau de pas. Les sequences d'un pattern sont exécutées en parallèle, les sequences sont ce que j'ai déjà défini plus bas]

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

A theLanguage program is a sequence of _instructions_ (@lst:instruction) that can either be _control_ instructions (a list of all the control instructions is given in @sec:control) or _effect_ instructions (a list of all the effect instructions is given in @sec:effect).

#figure([
  #set align(left)
  #raw("pub enum Instruction {
    Control(ControlASM),
    Effect(Event, TimeSpan),
}")
  ],
  caption: "Instruction definition"
) <lst:instruction>

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

In theLanguage programs one refers to variables by their name but also has to explicitly state their kind.
Therefore, there is no issue with variables of different kinds having the same name.

/*
In the case where several variables with the same name exist, the one with the smallest scope is used.
In other words: 
- if there is an ephemeral variable $v$ declared by some instruction $i$ in a theLanguage program and there exists an environment variable, a global variable, or a persistent variable also called $v$, then any reading or writing to $v$ after the execution of $i$ will be on the ephemeral variable;
- if there is a persistent variable $v$ declared by some instruction $i$ in a theLanguage program and there exists an environment variable or a global variable also called $v$, then any reading or writing to $v$ after the execution of $i$ will be on the persistent variable;
- if there is a global variable $v$ declared by som instruction $i$ in a theLanguage program and there exists an environment variable also called $v$, then any reading or writing to $v$ in any theLanguage program after the execution of $i$ will be on the global variable.
*/

== A few words on functions

#text(blue)[TODO: à écrire]

= theLanguage: theTool Intermediate Low-level Language

In this section we describe all the control instructions (@sec:control) and all the effect instructions (@sec:effect) available in the theLanguage language.
These instructions use variables and durations and we explain how they behave in @sec:variables and @sec:timing respectively.
We also list the environment variables (@sec:envvariables).

== Types of variables <sec:variables>

Each variable (being environment, global, persistent or ephemeral) and constant has a type.

=== Existing types

The possible types are given in @lst:types, which is an extract of the file ``` src/lang/variable.rs```.

#figure([
  #set align(left)
  #raw("pub enum VariableValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Func(Program),
    Duration(TimeSpan),
  }")
  ],
  caption: "Types"
) <lst:types>

#text(red)[TODO: je pense que ce serait bien d'uniformiser, genre Int, Float, Bool, Str, Func ou bien Integer, Floating, Boolean, String, Function. J'ai pris la première option, mais ce n'est peut-être pas possible en Rust si les types sont déjà utilisés ?]

#text(red)[TODO: je ne sais pas si c'est exactement comme ça qu'il faut rajouter le temps dans les types de variables]

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
  columns: 7,
  inset: 10pt,
  fill: (x, y) =>
    if x !=0 and x == y { 
      gray 
    } else if x == 0 or y == 0 {
      green.lighten(80%)
    },
  align: horizon,
  table.header(
    [*From\\To*], [*Int*], [*Float*], [*Bool*], [*Str*], [*Func*], [*Duration*],
  ),
  [*Int*], [], smallCell[Represented\ as float], smallCell[$0  arrow #false$\ $!= 0 arrow #true $], smallCell[Decimal\ representation], smallCell[$bot$], smallCell[Int as milliseconds],
  [*Float*], smallCell[Rounded\ to int], smallCell[], smallCell[$0  arrow #false$\ $!= 0 arrow #true $], smallCell[Decimal\ representation], smallCell[$bot$], smallCell[Rounded to int as milliseconds],
  [*Bool*], smallCell[$#false arrow 0$\ $#true arrow 1$], smallCell[$#false arrow 0.0$\ $#true arrow 1.0$], smallCell[], smallCell[$#false arrow$ "False"\ $#true arrow$ "True"], smallCell[$bot$], smallCell[?],
  [*Str*], smallCell[Parsed as int\ (0 if error)], smallCell[Parsed as float\ (0 if error)], smallCell["" $arrow #false$ \ $!=$"" $arrow #true$], smallCell[], smallCell[$bot$], smallCell[Parsed as time duration (0 if error)],
  [*Func*], smallCell[$bot arrow 0$\ $!=bot arrow 1$], smallCell[$bot arrow 0.0$\ $!=bot arrow 1.0$], smallCell[$bot arrow #false$\ $!=bot arrow #true$], smallCell[Name of the\ function], smallCell[], smallCell[?],
  [*Duration*], smallCell[Milliseconds as int], smallCell[Milliseconds represented as float], smallCell[$0$ms $-> #false$\ $!=0$ms $-> #true$], smallCell[Time as string], smallCell[$bot$], smallCell[],
)
) <tab:casting>


== Control instructions <sec:control>

Control instructions allow to perform basic operations (boolean and arithmetic) over variables.
They also can change the control-flow of a program.

Concretely, a theLanguage program is a vector of instructions. 
At any time, the next instruction to be executed is given by a position in this vector (think of the program counter for a processor) that the scheduler stores.
After executing an instruction, by default this position is increased by one.
To alter the control-flow, a few instructions allow to arbitrarily change this position (jump instructions) or even to change the vector that represents the current program (call and return instructions).

The existing control instructions are given in @lst:asm, which is an extract of the file ``` src/lang/control_asm.rs```.


#figure([
  #set align(left)
  #raw("pub enum ControlASM {
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
    BitAnd(Variable, Variable, Variable),
    BitNot(Variable, Variable),
    BitOr(Variable, Variable, Variable),
    BitXor(Variable, Variable, Variable),
    ShiftLeft(Variable, Variable, Variable),
    ShiftRightA(Variable, Variable, Variable),
    ShiftRightL(Variable, Variable, Variable),
    // String operations
    Concat(Variable, Variable, Variable),
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
  }")
  ],
  caption: "Control instructions"
) <lst:asm>

=== Arithmetic operations

These instructions are all of the form ``` Op(x, y, z)```.
Arguments x and y are inputs and z is an output.
It is expected that x and y are two numbers of the same type (int, float or duration).
If this is not the case: 
- if z is a float, an int or a duration, they will both be casted to the type of z,
- else they will both be casted to the type of x.
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
  [BitAnd], [$z <- x \& y$], [],
  [BitNot], [$z <- ~ x $], [],
  [BitOr], [$z <- x | y$], [],
  [BitXor], [$z <- x \^ y$], [],
  [ShiftLeft], [$z <- x << y$], [],
  [ShiftRightA], [$z <- x >> y$], [arithmetic shift],
  [ShiftRightL], [$z <- x >> y$], [logical shift],
)) <tab:bitwise>

=== String operations

These instructions are all of the form ``` Op(x, y, z)```.
Arguments x and y are inputs and will be casted to str (if needed).
Argument z is an output.
The result of the operation will be casted to the type of z (if needed).

Each instruction performs a different operation, as shown in @tab:string.

#figure(
  caption: "String operations semantics",
table(
  columns: 3,
  inset: 10pt,
  align: horizon,
  table.header(
    [*Op*], [*Semantics*], [*Remark*],
  ),
  [Concat], [$z <- x.y$], [string concatenation],
)) <tab:string>

=== Memory manipulation

The three variable declaration instructions (DeclareEphemeral, DeclareGlobal, DeclarePersistent) are of the form ``` Declare(name, value)``` and will create a new (Ephemeral, Globale or Persistent respectively) variable named ``` name``` and initialize its value to ``` value```.
The type of the new variable is the type of ``` value```.

Notice that, in any program instruction arguments, if a variable that does not exists is used, it will be created with a 0 value.

The ``` mov(x, y)``` instruction semantics is $y <- x$.
If needed, the value of ``` x``` will be casted to the type of ``` y```.

=== Jumps

By default, the instructions of a theLanguage program are executed one after the other in the order in which they are stored in the vector representing the program.
At each time, the position of the instruction to be executed is stored by the scheduler (think of a program counter for a processor).
Assume that the place where this position is stored is called ``` pc```.
By default, after executing an instruction, the scheduler increases ``` pc```: $"pc" <- "pc"+1$.
Jump instructions allow to replace this standard update of ``` pc``` by something else, potentially based on a condition.

The semantics of the different jump instructions is given in @tab:jumps.
In each case, if the condition is $#true$ then $"pc" <- "pc" + d$.
Else, $"pc" <- "pc" + 1$.

#figure(
  caption: "Jumps semantics",
table(
  columns: 3,
  inset: 10pt,
  align: horizon,
  table.header(
    [*Instruction*], [*Cond.*], [*Remark*]
  ),
  [Jump(d)], [$#true$], [],
  [JumpIf(x, d)], [$x$], [$x$ casted to Bool],
  [JumpIfDifferent(x, y, d)], [$x != y$], [$y$ casted to the type of $x$],
  [JumpIfEqual(x, y, d)], [$x = y$], [$y$ casted to the type of $x$],
  [JumpIfLess(x, y, d)], [$x < y$], [$y$ casted to the type of $x$],
  [JumpIfLessOrEqual(x, y, d)], [$x <= y$], [$y$ casted to the type of $x$],
)) <tab:jumps>

=== Calls and returns

#text(red)[TODO: pas mal de trucs à rajouter dans le scheduler pour gérer ça]

A jump instruction always jumps to the same position in a program.
Hence, one cannot use them to simulate procedure calls (the return position from a procedure depends of the point in code at which the jump to the procedure happened).

Calls are jumps that store, in a stack, the position from which they jumped.
Returns are jumps that read in this stack to determine the position to which they jump.
This stack will be called _return stack_.

*CallFunction(f).* Cast $f$ to a program, then replace the current program $p$ with $f$. Push ($p, "pc" + 1)$ into the return stack. Set ``` pc``` to 0 (the start of the new program).

*CallProcedure(pos).* Push $(p, "pc" + 1)$ (where $p$ is the current program) into the return stack. Set ``` pc``` to pos.

*Return.* Pop $(p, "pos")$ from the return stack. Replace the current program with $p$ (if needed) and set ``` pc``` to pos.


== Effect instructions <sec:effect>

Effect instructions are constituted of an _Event_ and a _TimeSpan_ (@lst:instruction).
The Event describes the effect of the instruction on the World and the TimeSpan tells how much time shall elapse after the event occurs.

The existing events are given in @lst:event, which is an extract of the file ``` src/lang/event.rs```.

In this section we give the semantics of these events.

#figure([
  #set align(left)
  #raw("pub enum Event {
    // Meta
    Nop,
    List(Vec<Event>),
    // Music
    PlayChord(Vec<Variable>, Variable),
    // Time handling
    SetBeatDuration(Variable),
    SetStepDuration(Variable, Variable),
    // Program starting
    ContinueAll,
    ContinueInstance(Variable),
    ContinueOldest(Variable),
    ContinueStep(Variable),
    ContinueStepOldest(Variable, Variable),
    ContinueStepYoungest(Variable, Variable),
    ContinueYoungest(Variable),
    Start(Variable, Variable),
    // Program halting
    PauseAll,
    PauseInstance(Variable),
    PauseOldest(Variable),
    PauseStep(Variable),
    PauseStepOldest(Variable, Variable),
    PauseStepYoungest(Variable, Variable),
    PauseYoungest(Variable),
    StopAll,
    StopInstance(Variable),
    StopOldest(Variable),
    StopStep(Variable),
    StopStepOldest(Variable, Variable),
    StopStepYoungest(Variable, Variable),
    StopYoungest(Variable),
}")
  ],
  caption: "Event definition"
) <lst:event>

=== Meta events

*Nop.* Does nothing.

*List(e).* Performs all the events in $e$ as fast as possible, in the order of the list.

=== Music events

Music events are the events that actually allow to play sound on a given device.
Not all devices accept all events.

*PlayChord(notes, d).* Plays all the notes given in _notes_ (casted to int used as midi values) together for $d$ (casted to a duration) milliseconds.

=== Time handling events

Time handling events allow to manage the relations between beats, step duration, and absolute time.

*SetBeatDuration(t).* Sets the duration of one beat to $t$ (casted to a duration). This duration is set in milliseconds (absolute time) by first evaluating $t$ in milliseconds. The standard use is to give $t$ in milliseconds to setup a tempo. However, one could give $t$ in beats for relative change of tempo (if $t$ is 3 beats the tempo is divided by 3 as the duration of a beat is multiplied by 3).

*SetStepDuration(n, t).* Sets the duration of step $n$ (casted to an int) to $t$ (casted to a duration). This duration is set in beats if possible or, else, it is set in milliseconds. The standard use is to give $t$ in beats, so that if beat duration changes step duration changes accordingly. However one could give $t$ in milliseconds to avoid this side effect. See @sec:envvariables for knowing how to get step numbers.

=== Program starting events <sec:starting>

Starting events allow to initiate new program instances (_start_) and to resume execution of program instances that were previously paused (_continue_).
How program instances can be paused is described in @sec:halting.

*ContinueAll.* Resumes all currently paused program instances.

*ContinueInstance(n).* Resumes the program instance with number $n$ (casted to an int). See @sec:envvariables for knowing how to get instance numbers.

*ContinueOldest(k).* Resumes the $k$ (casted to an int) program instances that were paused the longest time ago.

*ContinueStep(n).* Resumes all currently paused program instances corresponding to step $n$ (casted to an int). See @sec:envvariables for knowing how to get step numbers.

*ContinueStepOldest(n, k).* Resumes the $k$ (casted to an int) program instances corresponding to step $n$ (casted to an int) that were paused the longest time ago. See @sec:envvariables for knowing how to get step numbers.

*ContinueStepYoungest(n, k).* Resumes the $k$ (casted to an int) program instances corresponding to step $n$ (casted to an int) that were paused the shortest time ago. See @sec:envvariables for knowing how to get step numbers.

*ContinueYoungest(k).* Resumes the $k$ (casted to an int) program instances that were paused the shortest time ago.

*Start(p, i).* Starts a new instance of program $p$. If $p$ is a function, then this function is used as a program. Else the program corresponding to step $p$ (casted to an int) is used. The number of the new instance is recorded in $i$ (after casting it to the type of $i$).

=== Program halting events <sec:halting>

Halting events are of two kinds: _stop_ events and _pause_ events.
Stop events will end the execution of a (set of) program(s) instance(s). 
Pause events will pause the execution of a (set of) program(s) instance(s) allowing to continue their execution from the point at which they where paused using program starting events (@sec:starting).

We describe here the stop events as the corresponding pause events have the same behavior.

//*Stop.* Stops the program instance in which this event is used.

*StopAll.* Stops all the program instances currently running.

*StopInstance(n).* Stops the program instance with number $n$ (casted to an int). See @sec:envvariables for knowing how to get instance numbers.

*StopOldest(k).* Stops the $k$ (casted to an int) oldest program instances (that started the longest time ago).

*StopStep(n).* Stops all the program instances corresponding to step number $n$ (casted to an int). See @sec:envvariables for knowing how to get step numbers.

*StopStepOldest(n, k).* Stops the $k$ (casted to an int) oldest program instances (that started the longest time ago) corresponding to step number $n$ (casted to an int). See @sec:envvariables for knowing how to get step numbers.

*StopStepYoungest(n, k).* Stops the $k$ (casted to an int) youngest program instances (that started the shortest time ago) corresponding to step number $n$ (casted to an int). See @sec:envvariables for knowing how to get step numbers.

*StopYoungest(k).* Stops the $k$ (casted to an int) youngest program instances (that started the shortest time ago).

== Timing operators <sec:timing>

// on peut mettre un entier en tant que ms, pulsations, pas, etc et la duration reste la plus précise possible (donc ne convertit rien en ms)

// faire des calculs sur les durées (en ms, pulsations, etc)
// une durée pourrait être un "calcul" genre un arbre de calcul

// en tout cas, il faut pouvoir convertir de n'importe quelle sorte vers n'importe quelle autre

// tempo => beat duration in ms

#figure([
  #set align(left)
  #raw("pub enum TimeSpan {
    Micros(SyncTime),
    Beats(f64),
    Steps(f64),
}")
  ],
  caption: "TimeSpan definition"
) <lst:timespan>

== Environment variables <sec:envvariables>

// temps
// instances en cours pour un pas
// nombre de pas
// numéro du pas courant
// numéro du pas associé à ce programme (celui où il a démarré)
// instances en pause
// numéro d'instance du programme courant

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