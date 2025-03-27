use lalrpop_util::lalrpop_mod;
lalrpop_mod!(dummygrammar, "/compiler/dummylang/dummygrammar.rs");

mod dummycompiler;
mod dummyast;

pub use dummycompiler::DummyCompiler;