// BaLi, Basically a Lisp
use lalrpop_util::lalrpop_mod;
lalrpop_mod!(the_grammar_of_bali, "/lang/bali/the_grammar_of_bali.rs");

pub mod bali_ast;
mod bali_compiler;

pub use bali_compiler::BaliCompiler;
