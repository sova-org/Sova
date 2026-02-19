# Concepts of Boinx

_Boinx_ is an interpreted, timed and declarative programming language to schedule events over a time-span in a "visual" way. Because it is declarative (and thus non-imperative), the order of lines do not impact the order of execution.

Looking at the code examples at the end of this document might help a lot in the understanding of _Boinx_.

## Anatomy of a program

A _Boinx_ program is a collection of *Outputs* and *Assignments*. There is no order of execution in a script, everything starts as soon as the program starts, and the *Outputs* run simultaneously.

These objects are made of *Compositions* of *BoinxItems*. 