use std::{cell::LazyCell, collections::BTreeMap};

use sova_core::vm::{Program, language::{LanguageDocumentation, LanguageElement, Reference, ReferenceEntry}, variable::VariableValue};

use crate::alisp::words::{context::CONTEXT_WORDS, control::CONTROL_WORDS, generative::GENERATIVE_WORDS, memory::MEMORY_WORDS};

pub mod generative;
pub mod memory;
pub mod context;
pub mod control;

pub struct Word {
    name: &'static str,
    description: &'static str,
    example: Option<&'static str>,
    prog: fn() -> Program
}

impl Word {

    pub fn function(&self) -> VariableValue {
        VariableValue::Func((self.prog)())
    }

}

pub fn default_dictionary() -> (BTreeMap<String, VariableValue>, Reference) {
    let words : [(&'static str, &[Word]) ; 4] = [ 
        ("Control", &CONTROL_WORDS), 
        ("Generative", &GENERATIVE_WORDS), 
        ("Context", &CONTEXT_WORDS), 
        ("Memory", &MEMORY_WORDS)
    ];
    let mut map = BTreeMap::new();
    let mut reference = Reference::new();
    for (name, collection) in words {
        for word in collection {
            map.insert(word.name.to_string(), word.function());
            let mut entry = ReferenceEntry::new(word.description).with_category(name);
            if word.example.is_some() {
                entry = entry.with_example(word.example.unwrap());
            }
            reference.insert(
                word.name.into(), 
                entry
            );
        }
    }
    (map, reference)
}
