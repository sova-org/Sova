mod shape;
use serde::{Deserialize, Serialize};
pub use shape::*;

mod modifier;
pub use modifier::*;

use crate::{clock::SyncTime, vm::variable::VariableValue};


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ValueGenerator {
    pub shape: GeneratorShape,
    pub modifiers: Vec<GeneratorModifier>,
    pub state: Box<VariableValue>,
    pub start_date: SyncTime
}