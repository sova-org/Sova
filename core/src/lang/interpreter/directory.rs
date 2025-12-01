use std::{collections::HashMap, fmt, sync::Arc};

use crate::{lang::interpreter::{Interpreter, InterpreterFactory, asm_interpreter::ASMInterpreterFactory}, log_error, scene::script::Script};

#[derive(Default)]
pub struct InterpreterDirectory {
    pub factories: HashMap<String, Arc<dyn InterpreterFactory>>,
    asm_factory: ASMInterpreterFactory,
}

impl InterpreterDirectory {

    pub fn new() -> Self {
        Self {
            factories: Default::default(),
            asm_factory: ASMInterpreterFactory,        
        }
    }

    pub fn available_interpreters(&self) -> impl Iterator<Item = &str> {
        self.factories.keys().map(String::as_str)
    }

    pub fn has_interpreter(&self, lang: &str) -> bool {
        self.factories.contains_key(lang)
    }

    pub fn add_factory(&mut self, factory : impl InterpreterFactory + 'static) {
        self.factories.insert(factory.name().into(), Arc::new(factory));
    }

    pub fn remove_factory(&mut self, name : &str) -> Option<Arc<dyn InterpreterFactory>> {
        self.factories.remove(name)
    }

    pub fn get_factory(&self, lang : &str) -> Option<Arc<dyn InterpreterFactory>> {
        self.factories.get(lang).map(Arc::clone)
    }

    pub fn get_interpreter(&self, script : &Script) -> Option<Box<dyn Interpreter>> {
        if script.is_compiled() {
            self.asm_factory.make_instance(script)
        } else if let Some(factory) = self.factories.get(script.lang()) {
            match factory.make_instance(script) {
                Ok(instance) => Some(instance),
                Err(err) => {
                    log_error!("Factory '{}' error: {err}", script.lang());
                    None
                }
            }
        } else {
            None
        }
    }

}

impl fmt::Debug for InterpreterDirectory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InterpreterDirectory")
            .field("factories", &self.factories.keys())
            .field("asm_factory", &self.asm_factory)
            .finish()
    }
}