use crate::lang::{interpreter::InterpreterDirectory, Transcoder};

pub struct LanguageCenter {
    pub transcoder: Transcoder,
    pub interpreter_directory: InterpreterDirectory
}