use sova_core::compiler::CompilationState;
use sova_core::scene::script::Script;
use sova_core::vm::Language;
use sova_core::vm::interpreter::{Interpreter, InterpreterFactory};

use super::interpreter::ForthInterpreter;

pub struct ForthInterpreterFactory;

impl Language for ForthInterpreterFactory {
    fn name(&self) -> &str {
        "forth"
    }
    fn version(&self) -> (usize, usize, usize) {
        (1,0,0)
    }
    fn documentation(&self) -> sova_core::vm::language::LanguageDocumentation {
        use sova_core::vm::language::{LanguageDocumentation, LanguageElement::*};
        let mut doc = LanguageDocumentation::default();
        doc.reference.insert(Word("dup".into()), "Duplicate top of stack".into());
        doc.reference.insert(Word("swap".into()), "Swap top two stack items".into());
        doc.reference.insert(Word("drop".into()), "Remove top of stack".into());
        doc.reference.insert(Word("over".into()), "Copy second item to top".into());
        doc.reference.insert(Word("rot".into()), "Rotate third item to top".into());
        doc.reference.insert(Word("+".into()), "Addition".into());
        doc.reference.insert(Word("-".into()), "Subtraction".into());
        doc.reference.insert(Word("*".into()), "Multiplication".into());
        doc.reference.insert(Word("/".into()), "Division".into());
        doc.reference.insert(Word(".".into()), "Print top of stack".into());
        doc.reference.insert(Word(":".into()), "Begin word definition — : name ... ;".into());
        doc
    }
}

impl InterpreterFactory for ForthInterpreterFactory {
    
    fn make_instance(&self, script: &Script) -> Result<Box<dyn Interpreter>, String> {
        Ok(Box::new(ForthInterpreter::new(script.content())))
    }

    fn check(&self, _script: &Script) -> CompilationState {
        // Parsed(None) indicates "checked and valid" without caching anything
        CompilationState::Parsed(None)
    }
}
