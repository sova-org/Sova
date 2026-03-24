//! # Bob Language
//!
//! A terse, Teletype-inspired DSL for musical event generation.
//! Designed for live coding with minimal syntax overhead.
//!
//! ## Philosophy
//!
//! - **Polish notation**: Operator before operands (`ADD 2 MUL 3 4` → 14)
//! - **Fixed arity**: Each operator consumes a known number of arguments
//! - **No parentheses**: Nesting is implicit from argument counts
//! - **Device-agnostic events**: Key-value maps that devices interpret
//!
//! ## Quick Example
//!
//! ```text
//! DEV 1;
//! RANGE 0 3 :
//!     >> [note: ADD 60 I vel: RRAND 80 120];
//!     WAIT 0.25
//! END
//! ```
//!
//! See [`bob_ast`] for the full syntax specification.

use lalrpop_util::lalrpop_mod;
lalrpop_mod!(bob_grammar, "/bob/bob_grammar.rs");

pub mod bob_ast;
mod bob_compiler;
mod bob_syntax;
mod compile_expr;
mod context;
mod emit;
mod emit_runtime;
mod operators;

pub use bob_compiler::BobCompiler;

#[cfg(test)]
mod tests;
