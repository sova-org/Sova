#show "theTool": "BuboCore"
#show "theLanguage": "BILL"
#show "bali": "BaLi"
#show "Boolean-Expr": "Bool-Expr"
#show "Arithmetic-Expr": "Arithm-Expr"
#show "Timing-Information": "Timing-Info"

#set par(justify: true)
#set heading(numbering: "1.1")
#set figure(placement: auto)


#set raw(lang: "lisp")

#let nt(body) = {
  text()[_$angle.l$#body$angle.r$_]
}

#let t(body) = {
  text()[*#body*]
}

#let title = [
  The grammar of bali
]

#let titlegraph = [
  #show raw: set par(leading: 0.1cm)
  #show raw: set text(8pt)
  #raw("
▗▄▄▄▖▐▌   ▗▞▀▚▖        ▄▄▄ ▗▞▀▜▌▄▄▄▄  ▄▄▄▄  ▗▞▀▜▌ ▄▄▄      ▄▄▄  ▗▞▀▀▘    ▗▄▄▖ ▗▞▀▜▌▗▖   ▄ 
  █  ▐▌   ▐▛▀▀▘       █    ▝▚▄▟▌█ █ █ █ █ █ ▝▚▄▟▌█        █   █ ▐▌       ▐▌ ▐▌▝▚▄▟▌▐▌   ▄ 
  █  ▐▛▀▚▖▝▚▄▄▖       █         █   █ █   █      █        ▀▄▄▄▀ ▐▛▀▘     ▐▛▀▚▖     ▐▌   █ 
  █  ▐▌ ▐▌          ▗▄▖                                         ▐▌       ▐▙▄▞▘     ▐▙▄▄▖█ 
                   ▐▌ ▐▌                                                                  
                    ▝▀▜▌                                                                  
                   ▐▙▄▞▘                                                                  
  ",
  block: true)
]

#set page(
  footer: align(
    center + horizon,
    title
  ),
)


#align(center)[
  #titlegraph
  //#text(17pt)[*A basic Lisp inspired language*]
]


*Abstract.* bali is a small language inspired by the syntax of Lisp.
It has been developed in order to test theTool in Live-Coding situations.
This document presents the grammar of this language and gives insights on its semantics.

\

= The grammar

== The grammar itself

In the below grammar, a #t([Number]) is any line of one or more digits (ASCII characters 48 to 57).
An #t([Identifier]) is any line of one or more letters (ASCII characters 65 to 90 and 97 to 122) and - and \# characters, starting with a letter.

#grid(
  columns: 3,
  align: (left, right, left),
  column-gutter: 7pt,
  row-gutter: 7pt,
  [#nt([Program])], [::=], [#nt([Program]) #nt([Time-Statement]) | #nt([Time-Statement])],
  [#nt([Context])], [::=], [#nt([Context]) #nt([Context-Element]) | #nt([Context-Element])],
  [#nt([Context-Element])], [::=], [ch: #nt([Arithmetic-Expr]) | dev: #nt([Arithmetic-Expr]) | dur: #nt([Abstract-Fract]) | v: #nt([Arithmetic-Expr])], 
  [#nt([Timing-Information])], [::=], [#nt([Concrete-Fract]) | #nt([Concrete-Fract])\.f],
  [#nt([Time-Statement])], [::=], [(> #nt([Timing-Information]) #nt([Context]) #nt([Program]) ) | (>> #nt([Context]) #nt([Program]) )],
  [], [|], [(< #nt([Timing-Information]) #nt([Context]) #nt([Program]) ) | (<< #nt([Context]) #nt([Program]) )],
  [], [|], [(spread #nt([Timing-Information]) #nt([Context]) #nt([Program]) )],
  [], [|], [(loop #t([Number]) #nt([Timing-Information]) #nt([Context]) #nt([Program]) )],
  [], [|], [(eucloop #t([Number]) #t([Number]) #nt([Timing-Information]) #nt([Context]) #nt([Program]) )],
  [], [|], [(binloop #t([Number]) #t([Number]) #nt([Timing-Information]) #nt([Context]) #nt([Program]) )],
  [], [|], [(pick #nt([Arithmetic-Expr]) #nt([Context]) #nt([Program]) )],
  [], [|], [(? #nt([Number]) #nt([Context]) #nt([Program]) )],
  [], [|], [(alt #nt([Context]) #nt([Program]) )],
  [], [|], [(with #nt([Context]) #nt([Program]))],
  [], [|], [#nt([Control-Effect])],
  [#nt([Control-Effect])], [::=], [(seq #nt([Context]) #nt([Control-List]) ) | (if #nt([Boolean-Expr]) #nt([Context]) #nt([Control-List]) )],
  [], [|], [(for #nt([Boolean-Expr]) #nt([Context]) #nt([Control-List]) )],
  [], [|], [(pick #nt([Arithmetic-Expr]) #nt([Context]) #nt([Control-List]) )],
  [], [|], [(? #nt([Number]) #nt([Context]) #nt([Control-List]) )],
  [], [|], [(alt #nt([Context]) #nt([Control-List]) )],
  [], [|], [(with #nt([Context]) #nt([Control-List]))],
  [], [|], [#nt([Effect])],
  [#nt([Control-List])], [::=], [#nt([Control-List]) #nt([Control-Effect]) | #nt([Control-Effect])],
  [#nt([Effect])], [::=], [(def #t([Identifier]) #nt([Arithmetic-Expr]) )],
  [], [|], [(note #nt([Arithmetic-Expr]) #nt([Context]))],
  [], [|], [(prog #nt([Arithmetic-Expr]) #nt([Context]) )],
  [], [|], [(control #nt([Arithmetic-Expr]) #nt([Arithmetic-Expr]) #nt([Context]) )],
  [#nt([Concrete-Fract])], [::=], [(/\/ #t([Number]) #t([Number]) ) | #t([Number]) | #t([Decimal])],
  [#nt([Abstract-Fract])], [::=], [(/\/ #nt([Arithmetic-Expr]) #nt([Arithmetic-Expr]) ) | #nt([Arithmetic-Expr]) | #t([Decimal])],
  [#nt([Boolean-Expr])], [::=], [(and #nt([Boolean-Expr]) #nt([Boolean-Expr]) ) | (or #nt([Boolean-Expr]) #nt([Boolean-Expr]) )],
  [], [|], [(not #nt([Boolean-Expr]) )],
  [], [|], [(lt #nt([Arithmetic-Expr]) #nt([Arithmetic-Expr]) ) | (leq #nt([Arithmetic-Expr]) #nt([Arithmetic-Expr]) )],
  [], [|], [(gt #nt([Arithmetic-Expr]) #nt([Arithmetic-Expr]) ) | (geq #nt([Arithmetic-Expr]) #nt([Arithmetic-Expr]) )],
  [], [|], [(== #nt([Arithmetic-Expr]) #nt([Arithmetic-Expr]) ) | (!= #nt([Arithmetic-Expr]) #nt([Arithmetic-Expr]) )],
  [#nt([Arithmetic-Expr])], [::=], [(+ #nt([Arithmetic-Expr]) #nt([Arithmetic-Expr]) ) | (#sym.ast.op #nt([Arithmetic-Expr]) #nt([Arithmetic-Expr]) )],
  [], [|], [(- #nt([Arithmetic-Expr]) #nt([Arithmetic-Expr]) ) | (/ #nt([Arithmetic-Expr]) #nt([Arithmetic-Expr]) )],
  [], [|], [(% #nt([Arithmetic-Expr]) #nt([Arithmetic-Expr]) )],
  [], [|], [#t([Identifier]) | #t([Number])],
  [], [|], [(scale #nt([Arithm-Expr]) #nt([Arithm-Expr]) #nt([Arithm-Expr]) #nt([Arithm-Expr]) #nt([Arithm-Expr]))],
  [], [|], [(clamp #nt([Arithm-Expr]) #nt([Arithm-Expr]) #nt([Arithm-Expr]))],
  [], [|], [(min #nt([Arithm-Expr]) #nt([Arithm-Expr]))],
  [], [|], [(max #nt([Arithm-Expr]) #nt([Arithm-Expr]))],
  [], [|], [(quantize #nt([Arithm-Expr]) #nt([Arithm-Expr]))],
  [], [|], [(sine #nt([Arithm-Expr]))],
  [], [|], [(saw #nt([Arithm-Expr]))],
  [], [|], [(triangle #nt([Arithm-Expr]))],
  [], [|], [(isaw #nt([Arithm-Expr]))],
  [], [|], [(randstep #nt([Arithm-Expr]))],
)

== Reserved identifiers

A few identifiers are reserved.

*Musical notation.* 
All the identifier of the following form are reserved #footnote[With the exception of cb-2, c-2b, g\#8, g8\#, a8, ab8, a8b, a\#8, a8\#, b8, bb8, b8b, b\#8, b8\#.]: X, XY, Xb, XbY, XYb, X\#, X\#Y, XY\# with X a letter in ${c, d, e, f, g, a, b}$ and Y a natural number in $[-2, 8]$.  
For example, identifiers c, eb, f\#, gb7 and a-1\# are reserved.

*Global variables.*
The identifiers A, B, C, D, W, X, Y, and Z are reserved.

*Environment variables.*
The identifiers T and R are reserved.

== Syntax simplifications

For fractions one can always write (X /\/ Y) instead of ```(// X Y)``` in any bali program.

#nt([Context]) can be empty in all constructions using it, except for ```(with ...)```.

== Comments

At any point in a program, the symbol ; will start a comment.
This comment ends at the end of the line.

= The semantics

A bali program is associated to a frame (and thus a line and a scene) in theTool.
Each timing information used in bali is relative to this frame. 

== #t([Number]) and #t([Identifier])

A #t([Number]) is any 7 bits number (so, in [0, 128[). 
In case a number $n$ out of this range is used in a program the actual number that will be considered is $n mod 128$. 

The *musical notation* reserved identifiers represent notes as handled by Midi, that is numbers: c-2 is 0, g8 is 127, c3 is 60, c\#3 (or c3\#) is 61, cb3 (or c3b) is 59.
The letter gives the note in alphabetical notation.
The number gives the octave.
Omitting the number is similar to using 3: c is c3, eb is eb3.
They can be used exactly as numbers, they cannot be redefined.

The *environment variables* reserved identifier represent values that can change over time and are set by theTool.
Environment variable T represents the current beats per minute.
Environment variable R is a random number (in [0, 128[) determined by theTool each time R is used.
It is not possible to redefine these variables (with def, see below) and trying to do so will fail silently.

Appart from that, an #t([Identifier]) is a name for a variable that will hold a number.
They hold only numbers in [0, 128[.
In case a number $n$ out of this range is stored in a variable the actual number that will be used is $n mod 128$.
A variable is private to one program (in theTool several programs can execute at the same time) except for the *global variables* (reserved identifiers) that are shared between all programs. 

== #nt([Arithmetic-Expr])

An #nt([Arithmetic-Expr]) represents an arithmetic calculus over integer numbers in [0, 128[.
The result is always in [0, 128[.
If needed, a modulo is performed.

Available operators are: + (addition), #sym.ast.op (multiplication), - (subtraction), / (division), % (modulo).

The expression ``` (op a b)``` corresponds to the calculus $a op b$, that is ``` (% a b)``` corresponds to $a mod b$.

Additional utility functions are available:
- ```(scale val old_min old_max new_min new_max)```: Linearly maps _val_ from the range [_old_min_, _old_max_] to the range [_new_min_, _new_max_]. The result is clamped to the new range.
- ```(clamp val min max)```: Clamps _val_ to be within the range [_min_, _max_].
- ```(min a b)```: Returns the smaller of _a_ and _b_.
- ```(max a b)```: Returns the larger of _a_ and _b_.
- ```(quantize val step)```: Rounds _val_ to the nearest multiple of _step_.

Several stateful oscillator functions generate periodic signals commonly used in LFOs (Low-Frequency Oscillators). They return MIDI-compatible integer values in the range [1, 127]. Their _speed_ argument determines the frequency in cycles per beat. They maintain internal state (phase and last update time) across calls within the same script instance, ensuring smooth, continuous oscillation based on the elapsed beats.
- ```(sine speed)```: Generates a sine wave.
- ```(saw speed)```: Generates a sawtooth wave (ramping up).
- ```(triangle speed)```: Generates a triangle wave.
- ```(isaw speed)```: Generates a reverse sawtooth wave (ramping down).
- ```(randstep speed)```: Generates a stepped random signal. A new random value (1-127) is chosen at the beginning of each cycle (determined by _speed_) and held constant until the next cycle begins.

== #nt([Boolean-Expr])

A #nt([Boolean-Expr]) represents a boolean calculus over booleans and integer numbers in [0, 128[.
As expressed by the grammar: such an expression can be used only as a condition for a for loop or an if conditional.
In particular, the value resulting of the calculus corresponding to such an expression cannot be stored in a variable.

Available operators on booleans are: and, or, not.

Available operators on integers are: lt (strictly lower than), leq (lower or equal), gt (strictly greater than), geq (greater or equal), == (equal), != (not equal).

The expression ``` (op a b)``` corresponds to the calculus $a op b$, that is ``` (get a b)``` corresponds to $a >= b$.

== #nt([Concrete-Fract]) and #nt([Abstract-Fract])

A #nt([Concrete-Fract]) or an #nt([Abstract-Fract]) is a fraction used for expressing time durations.
The fraction is converted to a floating point value at the last possible moment (that is, when theTool has to compute a timestamp).

In practice ``` (// n d)``` represents a fraction with numerator $n$ and denominator $d$.
The alternative definition of a fraction as a single number or arithmetic expression $n$ represents a fraction with a numerator of $n$ and a denominator $1$.

A #nt([Concrete-Fract]) represents a fraction that will be computed at compile time.
It is defined from numbers only.
An alternative definition exists for #nt([Concrete-Fract]) as a decimal number $f$.
It represents a fraction with numerator $n$ and denominator $d$ such that $f = n/d$.
A #nt([Concrete-Fract]) can always be omitted, in which case it will be considered as $1/1$.

An #nt([Abstract-Fract]) represents a fraction that will be computed at execution time.
It must be defined using the explicit fraction syntax `(// n d)` or `(n // d)` where `n` and `d` are #nt([Arithmetic-Expr]).

== #nt([Effect])

An #nt([Effect]) changes the state of the program or impacts the external world.
At the moment there are five effects.

``` (def v e)```
sets the value of variable $v$ to $e$.
Any variable has value 0 by default.

``` (note n c)```
Sends a MIDI Note On message followed by a corresponding Note Off message after a specified duration. It targets a specific MIDI device. 
$n$ is the note number.
A velocity, a MIDI channel and a duration and the target device are obtained from the context $c$ if they are defined in it or, else, from the context in which this effect is used.

``` (prog p c)```
Sends a MIDI Program Change message to a specific MIDI device.
$p$ is the program number.
A MIDI channel and the target device are obtained from the context $c$ if they are defined in it or, else, from the context in which this effect is used.

``` (control con v c)```
Sends a MIDI Control Change message to a specific MIDI device.
_con_ is the control number and $v$ is the control value.
A MIDI channel and the target device are obtained from the context $c$ if they are defined in it or, else, from the context in which this effect is used.

== #nt([Control-Effect]) and #nt([Control-List])

A #nt([Control-Effect]) allows to perform #nt([Effect]) (or #nt([Control-Effect])) in sequence (seq), in loop (for), conditionally (if), or in a given context (with).
A #nt([Control-List]) is simply an ordered set of #nt([Control-Effect]).

``` (seq c s)``` will execute in order the elements of $s$ in the context $c$.

``` (if cond c s)``` will execute the elements of $s$ (not necessarily in order) in the context $c$ if the condition _cond_ is evaluated to #true.

``` (for cond c s)``` will execute all the elements in $s$ in the context $c$ as long as the condition _cond_ is evaluated to #true. One should avoid making infinite loops as this will mess with the timing requirements (see next section) due to theTool program execution model.

``` (pick expr c s)``` will evaluate the expression _expr_ to get a number $e$.
Then, the element number $e$ (modulo the length of $s$) in $s$ will be executed.

``` (? n c s)``` will execute $n$ randomly chosen elements of $s$ in the context $c$.
The $n$ elements will be different (if $n$ is greater than the number of elements, then each element is executed once).
It is possible to omit $n$, in which case it will be set to 1.

``` (alt c s)``` will execute one element of $s$ each time this effect is reached.
The elements of $s$ are executed in order.
This is also valid between scripts instances.
So, the script ``` (alt (note c) (note d))``` will play a C at its first execution, a D at its second execution, then a C, and so on.

``` (with c s)``` will execute the elements of $s$ (not necessarily in order) in the context $c$.

== #nt([Time-Statement])

A #nt([Time-Statement]) allows to perform some (list of) #nt([Control-Effect]) at a given time point.
The time is expressed as a #nt([Concrete-Fract]) because having variables here would lead to execution orders that cannot be decided at compile time.
The time points calculations are relative to the duration of a time window.
The time point is initially set to the beginning of the frame in which the program is executed.
The time window is initially set to the duration of the frame in which the program is executed.
The time point and the time window evolve with some of the statements.

=== Basic semantics

In the below descriptions of statements, we consider that _TW_ is the duration of the time window when the statement is used and _TP_ is the time point when the statement is used.

``` (> frac c p)``` executes $p$ in context $c$ at a point in time _frac_ $times$ _TW_ after _TP_.
So, intuitively, this shifts $p$ by a fraction of the time window.

``` (< frac c p)``` executes $p$ in context $c$ at a point in time _frac_ $times$ _TW_ before _TP_.
In case $p$ should be executed before the start of the current frame, it will be executed at the start of this frame but before any other thing that should be executed at a later time (even before the start of the frame).

``` (>> c p)``` executes $p$ in context $c$ at _TP_, but just after everything else that should occur at this time point.

``` (<< c p)``` executes $p$ in context $c$ at _TP_, but just before everything else that should occur at this time point.

For example, the program ``` (> 0.5 p1 (<< p2) (>> p3)``` will execute _p1_, _p2_ and _p3_ all at $1/2$ of the frame, but in the following order: _p2_, then _p1_, then _p3_.

``` (spread frac c p)``` sets _TW_ as _frac_ $times$ _TW_. 
Then executes the statements of $p$ (in context $c$) distributed fairly in _TW_ starting from _TP_.
Each statement is executed with a new time window equal to $italic("TW") / italic("len(p)")$ where _len(p)_ is the number of statements in $p$.

``` (loop n frac c p)``` sets _TW_ as $italic("frac") times italic("TW")$.
Then executes $n$ times, distributed fairly in _TW_ starting from _TP_, the program $p$ in context $c$.
Each execution of the program has a time window set to $italic("TW") / n$.

``` (eucloop steps beats frac c p)``` sets _TW_ as $italic("frac") times italic("TW")$.
Then executes _steps_ times the program $p$ in context $c$.
This executions take place on the time points corresponding to an euclidean rhythm whose beats are fairly distributed in _TW_ starting from _TP_.
Each execution of the program has a time window set to $italic("TW") / italic("beats")$.

``` (binloop val beats frac c p)``` sets _TW_ as $italic("frac") times italic("TW")$.
Then considers the time points obtained by distributing _beats_ points fairly in _TW_ starting from _TP_.
At each of these time points the program $p$ (in context $c$) is executed or not, depending on the binary representation of _val_ over 7 bits.
For example, if $italic("val") = 6$ the representation of _val_ is $0000110$. 
Then, if $italic("beats") = 7$, $p$ will be executed at the fifth and the sixth time points, but not at the other time points.
If _beats_ is smaller than 7 then only the _beats_ most significant bits of _val_ are considered.
In the previous example, if $italic("beats") = 5$, we consider $00001$.
If _beats_ is larger than 7 then we loop over the representation of _val_.
In the previous example, if $italic("beats") = 12$, we consider $000011000001$.

``` (pick expr c p)``` is similar to the pick #nt([Control-Effect]).
The only thing to notice is that _expr_ is evaluated once and for all at the first _TP_ at which a statement of $p$ could be executed.

``` (? n c p)``` is similar to the ? #nt([Control-Effect]).

``` (alt c p)``` is similar to the alt #nt([Control-Effect]).

``` (with c p)``` is similar to the with #nt([Control-Effect]).

=== Alternative semantics

The statements that admit a #nt([Timing-Information]) in their arguments can have their time points computed relatively to the duration of the frame in which the script is executed rather than relatively to the duration of their time window.
For that, the timing information shall be given with a trailing $.f$ (for example ``` (> 0.5.f p)``` instead of ``` (> 0.5 p)```).

The statements which change the time window (spread, loop, eucloop, binloop) can be used by giving the duration of a step rather than the duration of the full statement by adding a ```:step``` argument.
For example ``` (loop 4 0.5:step p)``` will execute $p$ 4 times, each time for the duration of half the frame and ``` (loop 4 0.5 p)``` will execute $p$ 4 times in half a frame (so each execution of $p$ will be over $1/8$ of the frame). 