use std::collections::BTreeMap;

use sova_core::{compiler::{CompilationError, Compiler}, vm::Program};

#[derive(Debug)]
pub struct LuaCompiler {}

impl LuaCompiler {
    pub fn translate_bytecode(_bytes: Vec<u8>) -> Program {
        let prog = Vec::new();

        prog
    }
}

impl Compiler for LuaCompiler {
    fn name(&self) -> &str {
        "lua"
    }

    fn compile(&self, text: &str, _args: &BTreeMap<String, String>) -> Result<Program, CompilationError> {
        let compiler = mlua::Compiler::new();
        match compiler.compile(text) {
            Ok(bytes) => Ok(Self::translate_bytecode(bytes)),
            Err(s) => Err(CompilationError {
                lang: self.name().to_owned(),
                info: s.to_string(),
                from: 0,
                to: 0,
            })
        }
    }
}