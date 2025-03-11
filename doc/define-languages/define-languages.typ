#show "theTool": "BuboCore"
#show "theLanguage": "BILL"

#set par(justify: true)
#set heading(numbering: "1.1")
#set raw(lang: "rust")
#set figure(placement: auto)


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

#align(center)[
  #text(21pt)[*Messing Around with theLanguage*\ ]
  #text(17pt)[*#title*]
]

\

*Abstract.* theTool has been designed so that it is (relatively) simple for a user to define their own scripting language(s) to be used for Live-Coding the steps of a pattern.
The general idea is to write a compiler that will translate scripts to a low-level language – theLanguage – that is interpreted by the theTool scheduler.
This requires to know theLanguage and to understand how the theTool scheduler works, which is the object of this document.
At the end we also give a few guidelines on how to properly integrate a new scripting language into theTool.

\

= The theTool scheduler

== General overview

As show in @fig:overview, the scheduler is responsible for emitting (time-stamped) events. 
These events are mostly sent to the World, the interface between theTool and the different devices — hardware or software — that it controls.
They can also occasionally be sent to other parts of theTool.

For that the scheduler loops forever, executing sequences of steps (each taken into a finite set of steps).
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
    Effect(Event, Variable),
}")
  ],
  caption: "Instruction definition"
) <lst:instruction>

The effect instructions are the ones that generate emissions of events to the World.
Any effect instruction contains two parts: an event $e$ and a duration $d$ (in @lst:instruction this last part is represented by a Variable, this will be explained later).
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
In case a program shall execute an effect instruction but the time for the event emission has not yet been met, its turn is skipped (so it does not pause all the program executions).

== Pattern, sequences, steps and some vocabulary

For the moment, we abstracted the exact way in which theTool scheduler handles steps.
The idea is that there is an object that we call a _pattern_ which is an array of objects called _sequences_. 
Each of these sequences is itself an array of _steps_.
A step is constituted of a theLanguage program (that we call the program _associated_ to this step) and a duration.

The theTool scheduler executes all the sequences in the pattern in parallel.
For executing a sequence it starts at the first step in the array.
Each steps is occurring for a time corresponding to its duration.
At the end of a step, the scheduler switches to the next step in the same sequence.
At the end of a sequence, the scheduler goes back to the start of this sequence.
At the beginning of any step, the scheduler starts an execution of the corresponding theLanguage program.
We call this execution an _instance_ of the program.

== How variables are handled

theLanguage programs can manipulate variables with control instructions and use them in effect instructions.
These variables are of five kinds: environment variables, global variables, sequence variables, step variables and instance variables (@lst:variables).


#figure([
  #set align(left)
  #raw("pub enum Variable {
    Environment(String),
    Global(String),
    Sequence(String),
    Step(String),
    Instance(String),
    Constant(VariableValue),
}")
  ],
  caption: "Kinds of variables"
) <lst:variables>

#text(red)[TODO: changer les noms dans le code]


=== Environment variables

From the point of view of theLanguage programs, environment variables are read-only variables.
Their values are set by the environment (think of time informations, random values, etc).
A list of these variables is given in @sec:envvariables.

=== Global variables

Global variables are shared among all the theLanguage program executions.

=== Sequence variables

Sequence variables are shared among all the theLanguage programs of a given sequence (the sequence in which they are declared).
They cannot be seen by programs associated to steps from other sequences.

=== Step variables

Step variables are shared among all the instances of the theLanguage program in which they are declared but are not seen by other programs.

=== Instance variables

Ephemeral variables are local to the instance of theLanguage program in which they are declared.
So, if several instances (parallel or not) of the same program exist, each of them has its own version of these variables.

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

Each variable (being environment, global, sequence, step, or instance) and constant has a type.

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
    Dur(TimeSpan),
  }")
  ],
  caption: "Types"
) <lst:types>

#text(red)[TODO: je pense que ce serait bien d'uniformiser, genre Int, Float, Bool, Str, Func ou bien Integer, Floating, Boolean, String, Function. J'ai pris la première option, mais ce n'est peut-être pas possible en Rust si les types sont déjà utilisés ?]

Integers, Float, Bool, Str and Dur variables are used to store values that can be read or written by the instructions of a program.

Func variables are programs themselves, they can be executed by calling them with the CallFunction control instruction. #text(red)[TODO: pas encore implanté]

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
    [*From\\To*], [*Int*], [*Float*], [*Bool*], [*Str*], [*Func*], [*Dur*],
  ),
  [*Int*], [], smallCell[Represented\ as float], smallCell[$0  arrow #false$\ $!= 0 arrow #true $], smallCell[Decimal\ representation], smallCell[$bot$], smallCell[Int as milliseconds],
  [*Float*], smallCell[Rounded\ to int], smallCell[], smallCell[$0  arrow #false$\ $!= 0 arrow #true $], smallCell[Decimal\ representation], smallCell[$bot$], smallCell[Rounded to int as milliseconds],
  [*Bool*], smallCell[$#false arrow 0$\ $#true arrow 1$], smallCell[$#false arrow 0.0$\ $#true arrow 1.0$], smallCell[], smallCell[$#false arrow$ "False"\ $#true arrow$ "True"], smallCell[$bot$], smallCell[?],
  [*Str*], smallCell[Parsed as int\ (0 if error)], smallCell[Parsed as float\ (0 if error)], smallCell["" $arrow #false$ \ $!=$"" $arrow #true$], smallCell[], smallCell[$bot$], smallCell[Parsed as time duration (0 if error)],
  [*Func*], smallCell[$bot arrow 0$\ $!=bot arrow 1$], smallCell[$bot arrow 0.0$\ $!=bot arrow 1.0$], smallCell[$bot arrow #false$\ $!=bot arrow #true$], smallCell[Name of the\ function], smallCell[], smallCell[?],
  [*Dur*], smallCell[Milliseconds as int], smallCell[Milliseconds represented as float], smallCell[$0$ms $-> #false$\ $!=0$ms $-> #true$], smallCell[Time as string], smallCell[$bot$], smallCell[],
)
) <tab:casting>

== Dealing with durations <sec:timing>

According to @lst:timespan (which is an extract of the file ``` src/clock.rs```), variables representing durations can hold three kinds of values: microseconds, beats, and steps.
A duration expressed as microseconds is an absolute time.
A duration expressed as beats is a relative time: the exact duration depends on the number of microseconds in a beat.
A duration expressed as steps is a relative time as well: the exact duration depends on the number of beats in the step associated to the theLanguage program in which the duration is used (that is, the step at which the program execution started).
The duration of a beat or a step can be changed by theLanguage programs and by the environment.


// en tout cas, il faut pouvoir convertir de n'importe quelle sorte vers n'importe quelle autre

// tempo => beat duration in ms

#figure([
  #set align(left)
  #raw("pub enum TimeSpan {
    Micros(u64),
    Beats(f64),
    Steps(f64),
}")
  ],
  caption: "TimeSpan definition"
) <lst:timespan>

Concrete durations are always expressed in microseconds. 
So, when a time-stamp must be associated to an event or when a delay must be applied the corresponding durations are converted to microseconds if needed.
Before that, durations are always kept as general as possible: when an arithmetic operation is performed between two durations, the most concrete one is converted to the kind of the most general, as show in @tab:duration.

#figure(
  caption: "Result kinds in arithmetic operations between durations",
  table(
  columns: 4,
  inset: 10pt,
  fill: (x, y) =>
    if x !=0 and x < y { 
      gray 
    } else if x == 0 or y == 0 {
      green.lighten(80%)
    },
  align: horizon,
  table.header(
    [], [*microseconds*], [*beats*], [*steps*],
  ),
  [*microseconds*], [microseconds], [beats], [steps],
  [*beats*], [], [beats], [steps],
  [*steps*], [], [], [steps],
)
) <tab:duration>

Sometimes, one may want the result of a computation on durations not to be as general as possible, e.g to be evaluated as microseconds immediately, to prevent the duration to change with changes to the beat duration or to a step duration.
For that, we provide operations to change the concreteness of a duration in @sec:control.

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
    Mod(Variable, Variable, Variable),
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
    // Time manipulation
    AsBeats(Variable, Variable),
    AsMicros(Variable, Variable),
    AsSteps(Variable, Variable),
    // Memory manipulation
    DeclareGlobale(String, Variable),
    DeclareInstance(String, Variable),
    DeclareSequence(String, Variable),
    DeclareStep(String, Variable),
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
It is expected that x and y are two numbers of the same type (Int, Float or Dur).
If this is not the case: 
- if z is a float, an int or a duration, they will both be casted to the type of z,
- else if x is a number y will be casted to the type of x,
- else if y is a number x will be casted to the type of y,
- else they will both be casted to Int.
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
  [Mod], [$z <- x mod y$], [$z <- x$ if $y = 0$],
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

=== Time manipulation

These instructions allow to perform conversions on durations.

*AsMicros(d, v).* Casts $d$ to a duration. Set this duration to microseconds, cast it to the type of $v$, and then store it in $v$.

*AsBeats(d, v).* Casts $d$ to a duration. Set this duration to beats, cast it to the type of $v$, and then store it in $v$.

*AsSteps(d, v).* Casts $d$ to a duration. Set this duration to steps, cast it to the type of $v$, and then store it in $v$.

=== Memory manipulation

The four variable declaration instructions (DeclareGlobal, DeclareInstance, DeclareSequence, DeclareStep) are of the form ``` Declare(name, value)``` and will create a new (Global, Instance, Sequence or Step) variable named ``` name``` and initialize its value to ``` value```.
The type of the new variable is the type of ``` value```.

Notice that, in any program instruction arguments, if a variable that does not exists is used, it will be created with a 0 value (except if it is an environment variable: reading a non-existing environment variable will give a 0 value but will not create the variable, writing it will have no effect).

The ``` mov(x, y)``` instruction semantics is $y <- x$.
If needed, the value of ``` x``` will be casted to the type of ``` y```.

=== Jumps

By default, the instructions of a theLanguage program are executed one after the other in the order in which they are stored in the vector representing the program.
At each time, the position of the instruction to be executed is stored by the scheduler (think of a program counter for a processor).
Assume that the place where this position is stored is called ``` pc```.
By default, after executing an instruction, the scheduler increases ``` pc```: $"pc" <- "pc"+1$.
Jump instructions allow to replace this standard update of ``` pc``` by something else, potentially based on a condition.

The semantics of the different jump instructions is given in @tab:jumps.
In each case, if the condition is $#true$ then $"pc" <- d mod n$ (where $n$ is the number of instructions in the program).
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
Hence, one cannot use them to simulate procedure calls (the return position from a procedure depends on the point in code at which the jump to the procedure happened).

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
    SetCurrentStepDuration(Variable),
    SetStepDuration(Variable, Variable),
    // Program starting
    Continue,
    ContinueInstance(Variable),
    ContinueOldest(Variable),
    ContinueSequence(Variable),
    ContinueSequenceOldest(Variable),
    ContinueSequenceYoungest(Variable),
    ContinueStep(Variable),
    ContinueStepOldest(Variable, Variable),
    ContinueStepYoungest(Variable, Variable),
    ContinueYoungest(Variable),
    Start(Variable, Variable),
    // Program halting
    Pause,
    PauseInstance(Variable),
    PauseOldest(Variable),
    PauseSequence(Variable),
    PauseSequenceOldest(Variable, Variable),
    PauseSequenceYoungest(Variable, Variable),
    PauseStep(Variable),
    PauseStepOldest(Variable, Variable),
    PauseStepYoungest(Variable, Variable),
    PauseYoungest(Variable),
    Stop,
    StopInstance(Variable),
    StopOldest(Variable),
    StopSequence(Variable),
    StopSequenceOldest(Variable, Variable),
    StopSequenceYoungest(Variable, Variable),
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

*List(e).* Performs all the events in $e$ as fast as possible (that is, kind of simultaneously it there are not too much events in $e$), in the order in which they are given.

=== Music events

Music events are the events that actually allow to play sound on a given device.
Not all devices accept all events.

*PlayChord(notes, d).* Plays all the notes given in _notes_ (casted to int used as midi values) together for $d$ (casted to a duration and set to milliseconds) milliseconds.

=== Time handling events

Time handling events allow to manage the relations between beats, step duration, and absolute time.

*SetBeatDuration(t).* Sets the duration of one beat to $t$ (casted to a duration). This duration is set in milliseconds (absolute time) by first evaluating $t$ in milliseconds. The standard use is to give $t$ in milliseconds to setup a tempo. However, one could give $t$ in beats for relative change of tempo (if $t$ is 3 beats the tempo is divided by 3 as the duration of a beat is multiplied by 3).

*SetCurrentStepDuration(t).* Sets the duration of the step associated to the program instance calling this instruction to $t$ (casted to a duration). This duration is set in beats if possible or, else, it is set in milliseconds. The standard use is to give $t$ in beats, so that if beat duration changes step duration changes accordingly. However one could give $t$ in milliseconds to avoid this side effect.

*SetStepDuration(n, t).* Same as SetCurrentStepDuration but for step $n$ (casted to an int). See @sec:envvariables for knowing how to get step numbers.

=== Program starting events <sec:starting>

Starting events allow to initiate new program instances (_start_) and to resume execution of program instances that were previously paused (_continue_).
How program instances can be paused is described in @sec:halting.

*Continue.* Resumes all currently paused program instances.

*ContinueInstance(n).* Resumes the program instance with number $n$ (casted to an int). See @sec:envvariables for knowing how to get instance numbers.

*ContinueOldest(k).* Resumes the $k$ (casted to an int) program instances that were paused the longest time ago.

*ContinueSequence(n).* Resumes all currently paused program instances corresponding to steps in sequence $n$ (casted to an int). See @sec:envvariables for knowing how to get sequence numbers.

*ContinueSequenceOldest(n, k).* Resumes the $k$ (casted to an int) program instances corresponding to steps in sequence $n$ (casted to an int) that were paused the longest time ago. See @sec:envvariables for knowing how to get sequence numbers.

*ContinueSequenceYoungest(n, k).* Resumes the $k$ (casted to an int) program instances corresponding to steps in sequence $n$ (casted to an int) that were paused the shortest time ago. See @sec:envvariables for knowing how to get sequence numbers.

*ContinueStep(n).* Resumes all currently paused program instances corresponding to step $n$ (casted to an int). See @sec:envvariables for knowing how to get step numbers.

*ContinueStepOldest(n, k).* Resumes the $k$ (casted to an int) program instances corresponding to step $n$ (casted to an int) that were paused the longest time ago. See @sec:envvariables for knowing how to get step numbers.

*ContinueStepYoungest(n, k).* Resumes the $k$ (casted to an int) program instances corresponding to step $n$ (casted to an int) that were paused the shortest time ago. See @sec:envvariables for knowing how to get step numbers.

*ContinueYoungest(k).* Resumes the $k$ (casted to an int) program instances that were paused the shortest time ago.

*Start(p, i).* Starts a new instance of program $p$. If $p$ is a function, then this function is used as a program. Else the program corresponding to step $p$ (casted to an int) is used. The number of the new instance is recorded in $i$ (after casting it to the type of $i$).
Remark that such a program instance is not associated to any step or sequence.
#text(blue)[TODO: est-ce que ça ne devrait pas être associé au step depuis lequel l'instruction est appelée ? ou alors pouvoir donner un step en paramètre ?]

=== Program halting events <sec:halting>

Halting events are of two kinds: _stop_ events and _pause_ events.
Stop events will end the execution of a (set of) program(s) instance(s). 
Pause events will pause the execution of a (set of) program(s) instance(s) allowing to continue their execution from the point at which they where paused using program starting events (@sec:starting).

We describe here the stop events as the corresponding pause events have the same behavior.

*Stop.* Stops all the program instances currently running.

*StopInstance(n).* Stops the program instance with number $n$ (casted to an int). See @sec:envvariables for knowing how to get instance numbers.

*StopOldest(k).* Stops the $k$ (casted to an int) oldest program instances (that started the longest time ago).

*StopSequence(n).* Stops all the program instances corresponding to steps in sequence number $n$ (casted to an int). See @sec:envvariables for knowing how to get sequence numbers.

*StopSequenceOldest(n, k).* Stops the $k$ (casted to an int) oldest program instances (that started the longest time ago) corresponding to steps in sequence number $n$ (casted to an int). See @sec:envvariables for knowing how to get sequence numbers.

*StopSequenceYoungest(n, k).* Stops the $k$ (casted to an int) youngest program instances (that started the shortest time ago) corresponding to steps in sequence number $n$ (casted to an int). See @sec:envvariables for knowing how to get sequence numbers.

*StopStep(n).* Stops all the program instances corresponding to step number $n$ (casted to an int). See @sec:envvariables for knowing how to get step numbers.

*StopStepOldest(n, k).* Stops the $k$ (casted to an int) oldest program instances (that started the longest time ago) corresponding to step number $n$ (casted to an int). See @sec:envvariables for knowing how to get step numbers.

*StopStepYoungest(n, k).* Stops the $k$ (casted to an int) youngest program instances (that started the shortest time ago) corresponding to step number $n$ (casted to an int). See @sec:envvariables for knowing how to get step numbers.

*StopYoungest(k).* Stops the $k$ (casted to an int) youngest program instances (that started the shortest time ago).


== Environment variables <sec:envvariables>

#text(blue)[TODO: on aurait envie d'avoir des variables d'environnement qui sont des ensembles, comment faire ? Ça demande sans doute d'ajouter un type de variable ? On voudrait aussi paramétrer les variables d'environnement mais en l'état ce n'est pas trop possible (par exemple pour obtenir le nombre de pas dans la séquence n), la version actuelle ne fonctionne pas vraiment car on ne peut pas construire les noms de variable dans un programme theLanguage. Il faut peut-être que les variables en question soient remplacées par des évènements (getters)]

#text(red)[TODO: à ajouter dans l'outil (mais pas tout de suite, il faut d'abord voir comment ça devrait marcher exactement)]

The environment variables provided by theTool are given below. 
Some of them are parameterized for simplicity. 
Parameters are depicted here between dollars signs, they should be replaced by integers.
For example, Sequence\$n\$NumSteps corresponds to the variables Sequence1NumSteps, Sequence2NumSteps, and so on.

- *InstanceID.* ID of this program instance.
- *Instance\$n\$SequenceID.* ID of the sequence containing the step associated to the program instance number $n$ (or the instance of the program using this variable if $n$ is omitted).
- *Instance\$n\$SequenceBeats.* Number of beats in the sequence containing the step associated to the program instance number $n$ (or the instance of the program using this variable if $n$ is omitted).
- *Instance\$n\$SequenceMicros.* Number of microseconds in the sequence containing the step associated to the program instance number $n$ (or the instance of the program using this variable if $n$ is omitted).
- *Instance\$n\$StepID.* ID of the step associated to the program instance number $n$ (or the instance of the program using this variable if $n$ is omitted).
- *Instance\$n\$StepBeats.* Number of beats in the step associated to the program instance number $n$ (or the instance of the program using this variable if $n$ is omitted).
- *Instance\$n\$StepMicros.* Number of microseconds in the step associated to the program instance number $n$ (or the instance of the program using this variable if $n$ is omitted).
- *Instance\$n\$SequenceNumInstances.* Same as NumInstances but only for instances corresponding to the sequence containing the step associated to the program instance number $n$ (or the instance of the program using this variable if $n$ is omitted).
- *Instance\$n\$SequenceNumRunning.* Same as NumRunning but only for instances corresponding to the sequence containing the step associated to the program instance number $n$ (or the instance of the program using this variable if $n$ is omitted).
- *Instance\$n\$SequenceNumPaused.* Same as NumPaused but only for instances corresponding to the sequence containing the step associated to the program instance number $n$ (or the instance of the program using this variable if $n$ is omitted).
- *Instance\$n\$StepNumInstances.* Same as NumInstances but only for instances corresponding to the step associated to the program instance number $n$ (or the instance of the program using this variable if $n$ is omitted).
- *Instance\$n\$StepNumRunning.* Same as NumRunning but only for instances corresponding to the step associated to the program instance number $n$ (or the instance of the program using this variable if $n$ is omitted).
- *Instance\$n\$StepNumPaused.* Same as NumPaused but only for instances corresponding to the step associated to the program instance number $n$ (or the instance of the program using this variable if $n$ is omitted).
- *NumInstances.* Number of instances currently running or paused.
- *NumPaused.* Number of instances currently paused.
- *NumRunning.* Number of instances currently running.
- *NumSequences.* Number of sequences.
- *Sequence\$n\$NumSteps.* Number of steps in sequence number $n$.
- *SequenceID.* ID of the sequence containing the step corresponding to this program.
- *Sequence\$n\$Beats.* Number of beats in the sequence number $n$ (or the sequence containing the step associated to the program using this variable if $n$ is omitted).
- *Sequence\$n\$Micros.* Number of microseconds in the sequence number $n$ (or the sequence containing the step associated to the program using this variable if $n$ is omitted).
- *Sequence\$n\$NumInstances.* Same as NumInstances but only for instances corresponding to the sequence number $n$ (or the sequence containing the step associated to the program using this variable if $n$ is omitted).
- *Sequence\$n\$NumRunning.* Same as NumRunning but only for instances corresponding to the sequence number $n$ (or the sequence containing the step associated to the program using this variable if $n$ is omitted).
- *Sequence\$n\$NumPaused.* Same as NumPaused but only for instances corresponding to the sequence number $n$ (or the sequence containing the step associated to the program using this variable if $n$ is omitted).
- *StepID.* ID of the step corresponding to this program.
- *Step\$n\$SequenceID.* ID of the sequence containing the step number $n$ (or the step associated to the program using this variable if $n$ is omitted).
- *Step\$n\$SequenceBeats.* Number of beats in the sequence containing the step number $n$ (or the step associated to the program using this variable if $n$ is omitted).
- *Step\$n\$SequenceMicros.* Number of microseconds in the sequence containing the step number $n$ (or the step associated to the program using this variable if $n$ is omitted).
- *Step\$n\$Beats.* Number of beats in the step number $n$ (or the step associated to the program using this variable if $n$ is omitted).
- *Step\$n\$Micros.* Number of microseconds in the step number $n$ (or the step associated to the program using this variable if $n$ is omitted).
- *Step\$n\$NumInstances.* Same as NumInstances but only for instances corresponding to the step number $n$ (or the step associated to the program using this variable if $n$ is omitted).
- *Step\$n\$NumRunning.* Same as NumRunning but only for instances corresponding to the step number $n$ (or the step associated to the program using this variable if $n$ is omitted).
- *Step\$n\$NumPaused.* Same as NumPaused but only for instances corresponding to the step number $n$ (or the step associated to the program using this variable if $n$ is omitted).
- *Step\$n\$SequenceNumInstances.* Same as NumInstances but only for instances corresponding to the sequence containing the step number $n$ (or the step associated to the program using this variable if $n$ is omitted).
- *Step\$n\$SequenceNumRunning.* Same as NumRunning but only for instances corresponding to the sequence containing the step number $n$ (or the step associated to the program using this variable if $n$ is omitted).
- *Step\$n\$SequenceNumPaused.* Same as NumPaused but only for instances corresponding to the sequence containing the step number $n$ (or the step associated to the program using this variable if $n$ is omitted).
- *TotalBeats.* Number of beats since the launch of theTool.
- *TotalMicros.* Number of microseconds since the launch of theTool. This cannot be computed from TotalBeats as the duration of a beat may have changed over time.
- *BeatMicros.* Number of microseconds in a beat. 

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

== Compiler as a standalone binary

#text(blue)[TODO: regarder comment ça marche et faire un exemple]