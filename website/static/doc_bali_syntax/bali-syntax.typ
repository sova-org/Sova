#show "theTool": "BuboCore"
#show "theLanguage": "BILL"
#show "bali": "BaLi"
#show "Boolean-Expr": "Bool-Expr"
#show "Arithmetic-Expr": "Arithm-Expr"

#set par(justify: true)
#set heading(numbering: "1.1")
#set figure(placement: auto)

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
All the identifier of the following form are reserved #footnote[With the exception of cb-2, c-2b, g\#8, g8\#, a8, ab8, a8b, a\#8, a8\#, b8, bb8, b8b, b\#8, b8\#.]: X, XY, Xb, XbY, XYb, X\#, X\#Y, XY\# with X a letter in ${c, d, e, f, g, a, b}$ and Y a natural number in $[-2, 8]$.  
For example, identifiers c, eb, f\#, gb7 and a-1\# are reserved.

== Syntax simplifications

For fractions one can always write (X /\/ Y) instead of (/\/ X Y) in any bali program.

Moreover, in (note n v c d), arguments v and c are optional: one can write (note n v d) and (note n d).
In these cases, c and v (if needed) will have default values.

= The semantics