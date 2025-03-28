// BaLi, Basically a Lisp
use lalrpop_util::lalrpop_mod;
lalrpop_mod!(the_grammar_of_bali, "/compiler/bali/the_grammar_of_bali.rs");

mod bali_compiler;
mod bali_ast;

pub use bali_compiler::BaliCompiler;