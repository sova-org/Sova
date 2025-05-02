use lalrpop_util::lalrpop_mod;
lalrpop_mod!(dummygrammar, "/compiler/dummylang/dummygrammar.rs");

mod dummyast;
mod dummycompiler;

pub use dummycompiler::DummyCompiler;
