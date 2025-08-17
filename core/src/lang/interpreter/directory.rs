use std::collections::HashMap;

use crate::{lang::interpreter::{asm_interpreter::ASMInterpreterFactory, Interpreter, InterpreterFactory}, scene::script::Script, transcoder::Transcoder};

#[derive(Default)]
pub struct InterpreterDirectory {
    pub factories: HashMap<String, Box<dyn InterpreterFactory>>,
    asm_factory: ASMInterpreterFactory,
}

impl InterpreterDirectory {

    pub fn new() -> Self {
        Self {
            factories: Default::default(),
            asm_factory: ASMInterpreterFactory,        
        }
    }

    pub fn has_interpreter(&self, lang: &str) -> bool {
        self.factories.contains_key(lang)
    }

    pub fn register_factory(&mut self, factory : impl InterpreterFactory + 'static) {
        self.factories.insert(factory.name().into(), Box::new(factory));
    }

    pub fn remove_factory(&mut self, name : &str) -> Option<Box<dyn InterpreterFactory>> {
        self.factories.remove(name)
    }

    pub fn get_interpreter(&self, script : &Script) -> Option<Box<dyn Interpreter>> {
        if let Some(factory) = self.factories.get(script.lang()) {
            Some(factory.make_instance(script))
        } else {
            self.asm_factory.make_instance(script)
        }
    }

}