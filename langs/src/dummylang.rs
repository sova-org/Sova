use lalrpop_util::lalrpop_mod;
lalrpop_mod!(dummygrammar, "/dummylang/dummygrammar.rs");

mod dummyast;
mod dummycompiler;

pub use dummycompiler::DummyCompiler;
