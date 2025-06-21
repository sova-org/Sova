use crate::compiler::bali::bali_ast::concrete_fraction::ConcreteFraction;
use crate::compiler::bali::bali_ast::expression::Expression;
use crate::lang::variable::Variable;

#[derive(Debug, Clone)]
pub enum Information {
    Alt(AltInformation),
    Choice(ChoiceInformation),
    Pick(PickInformation),
    Ramp(RampInformation),
}

#[derive(Debug, Clone)]
pub enum TimingInformation {
    FrameRelative(ConcreteFraction),
    PositionRelative(ConcreteFraction),
}

impl TimingInformation {
    pub fn as_frames(&self, spread_time: &ConcreteFraction) -> ConcreteFraction {
        match self {
            TimingInformation::FrameRelative(time) => time.clone(),
            TimingInformation::PositionRelative(time) => time.mult(spread_time),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RampInformation {
    pub variable_name: String, // nom de la variable de la rampe
    pub variable_value: i64,   // valeur de la variable à ce point de la rampe
}

#[derive(Debug, Clone)]
pub struct ChoiceInformation {
    pub variables: Vec<Variable>, // variables utilisée pour faire ce choix
    pub target_variables: Vec<Variable>, // variables utilisées pour stocker les valeurs visées pour les variables de choix
    //pub num_selectable: i64, // nombre d'éléments disponibles pour le choix
    pub position: usize, // position de cet élément particulier dans la liste des éléments du choix
}

#[derive(Debug, Clone)]
pub struct PickInformation {
    pub variable: Variable,     // variable utilisée pour ce pick
    pub position: usize,        // position de l'élément considéré dans le pick
    pub possibilities: usize,   // nombre d'éléments dans le pick
    pub expression: Expression, // expression pour obtenir la valeur du pick
    pub num_variable: i64,      // numéro de la variable dans l'ordre de génération
}

#[derive(Debug, Clone)]
pub struct AltInformation {
    pub frame_variable: Variable, // variable de frame utilisée pour ce alt
    pub instance_variable: Variable, // variable d'instance utilisée pour ce alt
    pub position: usize,          // position de l'élément considéré dans le alt
    pub possibilities: usize,     // nombre d'éléments dans le alt
    pub num_variable: i64,        // numéro de la variable dans l'ordre de génération
}
