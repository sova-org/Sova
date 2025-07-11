#show "theTool": "Sova"
#show "theLanguage": "BILL"
#show "bali": "BaLi"

#set par(justify: true)
#set heading(numbering: "1.1")
#set figure(placement: auto)


#set raw(lang: "rust")

#let title = [
  The Architecture of theTool
]

#let titlegraph = [
  #show raw: set par(leading: 0.1cm)
  #show raw: set text(8pt)
  #raw("
▗▄▄▄▖▐▌   ▗▞▀▚▖    ▗▞▀▜▌ ▄▄▄ ▗▞▀▘▐▌   ▄    ■  ▗▞▀▚▖▗▞▀▘   ■  █  ▐▌ ▄▄▄ ▗▞▀▚▖
  █  ▐▌   ▐▛▀▀▘    ▝▚▄▟▌█    ▝▚▄▖▐▌   ▄ ▗▄▟▙▄▖▐▛▀▀▘▝▚▄▖▗▄▟▙▄▖▀▄▄▞▘█    ▐▛▀▀▘
  █  ▐▛▀▚▖▝▚▄▄▖         █        ▐▛▀▚▖█   ▐▌  ▝▚▄▄▖      ▐▌       █    ▝▚▄▄▖
  █  ▐▌ ▐▌                       ▐▌ ▐▌█   ▐▌             ▐▌                 
                                          ▐▌             ▐▌                 
                                                                            
                                                                            
                      ▄▄▄  ▗▞▀▀▘     ▗▄▄▖ ▄▄▄  ▄   ▄ ▗▞▀▜▌                  
                     █   █ ▐▌       ▐▌   █   █ █   █ ▝▚▄▟▌                  
                     ▀▄▄▄▀ ▐▛▀▘      ▝▀▚▖▀▄▄▄▀  ▀▄▀                         
                           ▐▌       ▗▄▄▞▘                                      
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


*Abstract.* theTool is a live coding tool that has been originally developed by Tanguy Dubois, Raphaël Forment and Loïg Jezequel from an idea discussed between Raphaël, Loïg and Rémi Georges#footnote[This original development has been supported by Athénor CNCM and the Laboratoire des sciences du numérique de Nantes (LS2N).].
This document presents the design of theTool: not necessarily what exists in the current state of the tool, but what we are aiming for.
It should be used as a guideline for developers wanting to contribute to theTool.

\

= A bit of history

In winter 2024, Raphaël and Rémi have been invited by Athénor CNCM to meet researchers from the LS2N in Nantes.
One of the goals of this meeting was to investigate the possibility to propose activities around the concept of live coding for schools in Loire-Atlantique.
After discussions, Raphaël, Rémi, and Loïg agreed that it would be interesting to let the students of the schools to design their own live coding languages.
This would require to develop a new live coding tool that would allow to quickly implement new languages from the ideas of the students.
A few months later, this new tool would become theTool.

= The general modular design

The general idea of theTool is to have a very modular design, so that any part of it could be easily replaced: the graphical interface, the sound engines, the scheduler, etc, and, obviously, the live coding languages.

#figure(
  image("modular-design.png", width: 80%),
  caption: [The modular design of theTool],
) <fig:overview>