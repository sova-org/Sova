use std::collections::BTreeMap;

use crate::{compiler::{CompilationError, Compiler}, lang::Program};

#[derive(Debug)]
pub struct LuaCompiler {}

impl LuaCompiler {
    pub fn translate_bytecode(bytes: Vec<u8>) -> Program {
        let mut prog = Vec::new();

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