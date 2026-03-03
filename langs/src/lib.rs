pub mod bali;
pub mod bob;
pub mod boinx;
pub mod cagire;
pub mod rhai;

use sova_core::vm::{LanguageCenter, Transcoder, interpreter::InterpreterDirectory};

/// Single source of truth for all language registrations.
pub fn create_language_center() -> LanguageCenter {
    let mut transcoder = Transcoder::default();
    transcoder.add_compiler(bali::BaliCompiler);
    transcoder.add_compiler(bob::BobCompiler);

    let mut interpreters = InterpreterDirectory::new();
    interpreters.add_factory(boinx::BoinxInterpreterFactory);
    interpreters.add_factory(cagire::CagireInterpreterFactory);
    interpreters.add_factory(rhai::RhaiInterpreterFactory);

    LanguageCenter {
        transcoder,
        interpreters,
    }
}

#[cfg(test)]
mod tests {
    use super::create_language_center;

    #[test]
    fn registers_rhai_language() {
        let center = create_language_center();
        assert!(center.interpreters.has_interpreter("rhai"));
    }
}
