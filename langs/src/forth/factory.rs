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
        use sova_core::vm::language::{LanguageDocumentation, LanguageElement::*, ReferenceEntry};
        let mut doc = LanguageDocumentation::default();
        doc.reference.insert(Word("dup".into()), ReferenceEntry::new("Duplicate top of stack").with_example("5 dup ."));
        doc.reference.insert(Word("swap".into()), ReferenceEntry::new("Swap top two stack items").with_example("1 2 swap ."));
        doc.reference.insert(Word("drop".into()), ReferenceEntry::new("Remove top of stack").with_example("1 2 drop ."));
        doc.reference.insert(Word("over".into()), ReferenceEntry::new("Copy second item to top").with_example("1 2 over ."));
        doc.reference.insert(Word("rot".into()), ReferenceEntry::new("Rotate third item to top"));
        doc.reference.insert(Word("+".into()), ReferenceEntry::new("Addition").with_example("3 4 + ."));
        doc.reference.insert(Word("-".into()), ReferenceEntry::new("Subtraction"));
        doc.reference.insert(Word("*".into()), ReferenceEntry::new("Multiplication"));
        doc.reference.insert(Word("/".into()), ReferenceEntry::new("Division"));
        doc.reference.insert(Word(".".into()), ReferenceEntry::new("Print top of stack"));
        doc.reference.insert(Word(":".into()), ReferenceEntry::new("Begin word definition — : name ... ;").with_example(": double dup + ;\n5 double ."));
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
