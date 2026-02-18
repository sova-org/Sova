pub mod bali;
pub mod bob;
pub mod boinx;
pub mod cagire;
// pub mod dummylang;
// pub mod lua;
// pub mod rhai;

use std::collections::BTreeMap;
use sova_core::compiler::Compiler;

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
