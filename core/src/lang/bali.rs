// BaLi, Basically a Lisp
use lalrpop_util::lalrpop_mod;
lalrpop_mod!(bali_grammar, "/lang/bali/bali_grammar.rs");

pub mod bali_ast;
mod bali_compiler;

pub use bali_compiler::BaliCompiler;
