#show "theTool": "BuboCore"
#show "theLanguage": "BILL"
#show "bali": "BaLi"
#show "Boolean-Expr": "Bool-Expr"
#show "Arithmetic-Expr": "Arithm-Expr"

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

In the below grammar, a #t([Number]) is any sequence of one or more digits (ASCII characters 48 to 57).
An #t([Identifier]) is any sequence of one or more letters (ASCII characters 65 to 90 and 97 to 122) and - and \# characters, starting with a letter.

#grid(
  columns: 3,
  align: (left, right, left),
  column-gutter: 7pt,
  row-gutter: 7pt,
  [#nt([Program])], [::=], [#nt([Program]) #nt([Time-Statement]) | #nt([Time-Statement])],
  [#nt([Time-Statement])], [::=], [(> #nt([Concrete-Fract]) #nt([Program]) ) | (>> #nt([Program]) )],
  [], [|], [(< #nt([Concrete-Fract]) #nt([Program]) ) | (<< #nt([Program]) )],
  [], [|], [(loop #t([Number]) #nt([Concrete-Fract]) #nt([Program]) )],
  [], [|], [#nt([Control-Effect])],
  [#nt([Control-Effect])], [::=], [(seq #nt([Control-List]) ) | (if #nt([Boolean-Expr]) #nt([Control-List]) )],
  [], [|], [(for #nt([Boolean-Expr]) #nt([Control-List]) ) | #nt([Effect])],
  [#nt([Control-List])], [::=], [#nt([Control-List]) #nt([Control-Effect]) | #nt([Control-Effect])],
  [#nt([Effect])], [::=], [(def #t([Identifier]) #nt([Arithmetic-Expr]) )],
  [], [|], [(note #nt([Arithmetic-Expr]) #nt([Arithmetic-Expr]) #nt([Arithmetic-Expr]) #nt([Abstract-Fract]))],
  [], [|], [(prog #nt([Arithmetic-Expr]) #nt([Arithmetic-Expr]) )],
  [], [|], [(control #nt([Arithmetic-Expr]) #nt([Arithmetic-Expr]) #nt([Arithmetic-Expr]) )],
  [#nt([Concrete-Fract])], [::=], [(/\/ #t([Number]) #t([Number]) ) | #t([Number])],
  [#nt([Abstract-Fract])], [::=], [(/\/ #nt([Arithmetic-Expr]) #nt([Arithmetic-Expr]) ) | #nt([Arithmetic-Expr])],
  [#nt([Boolean-Expr])], [::=], [(and #nt([Boolean-Expr]) #nt([Boolean-Expr]) ) | (or #nt([Boolean-Expr]) #nt([Boolean-Expr]) )],
  [], [|], [(not #nt([Boolean-Expr]) )],
  [], [|], [(lt #nt([Arithmetic-Expr]) #nt([Arithmetic-Expr]) ) | (leq #nt([Arithmetic-Expr]) #nt([Arithmetic-Expr]) )],
  [], [|], [(gt #nt([Arithmetic-Expr]) #nt([Arithmetic-Expr]) ) | (geq #nt([Arithmetic-Expr]) #nt([Arithmetic-Expr]) )],
  [], [|], [(== #nt([Arithmetic-Expr]) #nt([Arithmetic-Expr]) ) | (!= #nt([Arithmetic-Expr]) #nt([Arithmetic-Expr]) )],
  [#nt([Arithmetic-Expr])], [::=], [(+ #nt([Arithmetic-Expr]) #nt([Arithmetic-Expr]) ) | (#sym.ast.op #nt([Arithmetic-Expr]) #nt([Arithmetic-Expr]) )],
  [], [|], [(- #nt([Arithmetic-Expr]) #nt([Arithmetic-Expr]) ) | (/ #nt([Arithmetic-Expr]) #nt([Arithmetic-Expr]) )],
  [], [|], [(% #nt([Arithmetic-Expr]) #nt([Arithmetic-Expr]) )],
  [], [|], [#t([Identifier]) | #t([Number])],
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

Moreover, in ``` (note n v c d)```, arguments v and c are optional: one can write ``` (note n v d)``` and ``` (note n d)```.
In these cases, c and v (if needed) will have default values.

== Comments

At any point in a program, the symbol ; will start a comment.
This comment ends at the end of the line.

= The semantics

A bali program is associated to a step (and thus a sequence and a pattern) in theTool.
Each timing information used in bali is relative to this step. 

== #t([Number]) and #t([Identifier])

A #t([Number]) is any 8 bits number (so, in [0, 128[). 
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
The alternative definition of a fraction — a single number or arithmetic expression $d$ — represents a fraction with a numerator of 1 and a denominator $d$ (except if $d = 0$ in which case the numerator is 0 and the denominator is 1).

A #nt([Concrete-Fract]) represents a fraction that will be computed at compile time.
It is defined from numbers only.

An #nt([Abstract-Fract]) represents a fraction that will be computed at execution time.
It can be defined from #nt([Arithmetic-Expr]).

== #nt([Effect])

An #nt([Effect]) changes the state of the program or impacts the external world.
At the moment there are four effects.

``` (def v e)```
sets the value of variable $v$ to $e$.
Any variable has value 0 by default.

``` (note n v c d)```
asks the default Midi device to play the note $n$ with velocity $v$ on channel $c$ for duration $d$.
The duration is an #nt([Abstract-Fract]).

``` (prog p c)```
sends a program change message to default Midi device.
With program $p$ on channel $c$.

``` (control con v c)```
sends a control change message to default Midi device.
With control _con_, value $v$, and on channel $c$.

== #nt([Control-Effect]) and #nt([Control-List])

A #nt([Control-Effect]) allows to perform #nt([Effect]) (or #nt([Control-Effect])) in sequence (seq), in loop (for), or conditionally (if).
A #nt([Control-List]) is simply an ordered set of #nt([Control-Effect]).

``` (seq s)``` will execute in order the elements of $s$.

``` (if cond s)``` will execute the elements of $s$ (not necessarily in order) if the condition _cond_ is evaluated to #true.

``` (for cond s)``` will execute all the elements in $s$ as long as the condition _cond_ is evaluated to #true. One should avoid making infinite loops as this will mess with the timing requirements (see next section) due to theTool program execution model.

== #nt([Time-Statement])

A #nt([Time-Statement]) allows to perform some (list of) #nt([Control-Effect]) at a given point in time.
The time is expressed as a #nt([Concrete-Fract]) because having variables here would lead to execution orders that cannot be decided at compile time.
The time is relative to the length of the step in which the program is executed.
It is possible to have nested #nt([Time-Statement]), in which case times are added.
The default time for executing something, when there is no #nt([Time-Statement]) is 0 (so, right at the beginning of the step).

``` (> frac p)``` executes $p$ at a point in time _frac_ after what was expected.

``` (< frac p)``` executes $p$ at a point in time _frac_ before what was expected.
In case $p$ should be executed at a negative time $t$, it will be executed at time 0 but before any other thing that should be be executed at time 0 or at a time negative but larger than $t$.

``` (>> p)``` executes $p$ at the expected time point, but just after everything else that should occur at this time point.

``` (<< p)``` executes $p$ at the expected time point, but just before everything else that should occur at this time point.

For example, the program ``` (> 5 p1 (<< p2) (>> p3)``` will execute _p1_, _p2_ and _p3_ all at $1/5$ of the step, but in the following order: _p2_, then _p1_, then _p3_.

Finally ``` (loop n frac p)``` executes $n$ times $p$.
First at the expected time point, then _frac_ after this point, then _frac_ later, and so on.

