use lalrpop_util::lalrpop_mod;
lalrpop_mod!(imp_grammar, "/lang/imp/imp_grammar.rs");

pub mod imp_ast;
mod imp_compiler;

pub use imp_compiler::ImpCompiler;
