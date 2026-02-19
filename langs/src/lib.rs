pub mod bali;
pub mod bob;
pub mod boinx;
pub mod cagire;

use std::collections::BTreeMap;
use sova_core::compiler::Compiler;
use sova_core::vm::{LanguageCenter, Transcoder, interpreter::InterpreterDirectory};

/// Single source of truth for all language registrations.
pub fn create_language_center() -> LanguageCenter {
    let mut transcoder = Transcoder::default();
    transcoder.add_compiler(bali::BaliCompiler);
    transcoder.add_compiler(bob::BobCompiler);

    let mut interpreters = InterpreterDirectory::new();
    interpreters.add_factory(boinx::BoinxInterpreterFactory);
    interpreters.add_factory(cagire::CagireInterpreterFactory);

    LanguageCenter { transcoder, interpreters }
}

pub fn try_compile(lang: &str, code: &str) -> Result<(), String> {
    match lang {
        "bob" => bob::BobCompiler.compile(code, &BTreeMap::new()).map(|_| ()).map_err(|e| e.info),
        "bali" => bali::BaliCompiler.compile(code, &BTreeMap::new()).map(|_| ()).map_err(|e| e.info),
        "cagire" => {
            let mut dict = std::collections::HashMap::new();
            cagire::compiler_check(code, &mut dict)
        },
        "boinx" => boinx::parse_boinx(code).map(|_| ()).map_err(|e| e.info),
        _ => Err(format!("unknown language: {lang}")),
    }
}
