use crate::lang::{interpreter::InterpreterDirectory, Transcoder};

#[derive(Debug)]
pub struct LanguageCenter {
    pub transcoder: Transcoder,
    pub interpreters: InterpreterDirectory,
}

impl LanguageCenter {

    pub fn languages(&self) -> impl Iterator<Item = &str> {
        self.transcoder.available_compilers().chain(self.interpreters.available_interpreters())
    }

}