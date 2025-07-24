use std::collections::HashMap;

use crate::{lang::interpreter::{asm_interpreter::ASMInterpreterFactory, Interpreter, InterpreterFactory}, transcoder::Transcoder};

#[derive(Default)]
pub struct InterpreterDirectory {
    pub factories: HashMap<String, Box<dyn InterpreterFactory>>,
    asm_factory: ASMInterpreterFactory
}

impl InterpreterDirectory {

    pub fn new(transcoder : Transcoder) -> Self {
        Self {
            factories: Default::default(),
            asm_factory: ASMInterpreterFactory { transcoder },
        }
    }

    pub fn set_transcoder(&mut self, transcoder : Transcoder) {
        self.asm_factory.transcoder = transcoder;
    }

    pub fn transcoder(&self) -> &Transcoder {
        &self.asm_factory.transcoder
    }

    pub fn transcoder_mut(&mut self) -> &mut Transcoder {
        &mut self.asm_factory.transcoder
    }

    pub fn register_factory(&mut self, factory : impl InterpreterFactory + 'static) {
        self.factories.insert(factory.name().into(), Box::new(factory));
    }

    pub fn remove_factory(&mut self, name : &str) -> Option<Box<dyn InterpreterFactory>> {
        self.factories.remove(name)
    }

    pub fn get_interpreter(&self, lang : &str, content : &str, args: HashMap<String, String>) -> Option<Box<dyn Interpreter>> {
        if let Some(factory) = self.factories.get(lang) {
            Some(factory.make_instance(content, args))
        } else {
            self.asm_factory.make_instance(lang, content)
        }
    }

}