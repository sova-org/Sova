use std::collections::HashMap;

use super::Interpreter;

pub trait InterpreterFactory : Send + Sync {

    fn name(&self) -> &str;

    fn make_instance(&self, content : &str, args: HashMap<String, String>) -> Box<dyn Interpreter>;

}