pub mod bali;
pub mod bob;
pub mod boinx;
pub mod cagire;
pub mod alisp;
pub mod hydra;

use sova_core::vm::{LanguageCenter, Transcoder, interpreter::InterpreterDirectory};

/// Single source of truth for all language registrations.
pub fn create_language_center() -> LanguageCenter {
    let mut transcoder = Transcoder::default();
    transcoder.add_compiler(bali::BaliCompiler);
    transcoder.add_compiler(bob::BobCompiler);

    let mut interpreters = InterpreterDirectory::new();
    interpreters.add_factory(boinx::BoinxInterpreterFactory);
    interpreters.add_factory(cagire::CagireInterpreterFactory::new());
    interpreters.add_factory(hydra::HydraInterpreterFactory);

    LanguageCenter {
        transcoder,
        interpreters,
    }
}
