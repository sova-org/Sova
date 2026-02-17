pub mod bali;
pub mod bob;
pub mod boinx;
// pub mod dummylang;
pub mod forth;
// pub mod lua;
// pub mod rhai;

use std::collections::BTreeMap;
use sova_core::compiler::Compiler;

pub fn try_compile(lang: &str, code: &str) -> Result<(), String> {
    match lang {
        "bob" => bob::BobCompiler.compile(code, &BTreeMap::new()).map(|_| ()).map_err(|e| e.info),
        "bali" => bali::BaliCompiler.compile(code, &BTreeMap::new()).map(|_| ()).map_err(|e| e.info),
        "forth" => { let _ = forth::ForthInterpreter::new(code); Ok(()) },
        "boinx" => boinx::parse_boinx(code).map(|_| ()).map_err(|e| e.info),
        _ => Err(format!("unknown language: {lang}")),
    }
}
