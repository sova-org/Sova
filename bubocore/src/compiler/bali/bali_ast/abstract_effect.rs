use crate::compiler::bali::bali_ast::{
    BaliContext, TopLevelEffect, Effect, AltVariableGenerator, Expression, Value
};

#[derive(Debug, Clone)]
pub enum EffectType {
    Definition,
    Note,
    ProgramChange,
    ControlChange,
    Osc,
    Dirt,
    Aftertouch,
    ChannelPressure,
}

pub struct AbstractEffect {
    pub concrete_type: EffectType,
    pub args: Vec<AbstractArg>,
}

#[derive(Debug, Clone)]
pub enum AbstractArg {
    Alt(Vec<AbstractArg>),
    Choice(Vec<AbstractArg>),
    List(Vec<AbstractArg>),
    Concrete(ConcreteArg),
}

#[derive(Debug, Clone)]
pub enum ConcreteArg {
    Expr(Box<Expression>),
    Litteral(Value),
}

impl ConcreteArg {
    pub fn to_value(&self) -> Value {
        match self {
            ConcreteArg::Litteral(v) => v.clone(),
            _ => Value::Number(0), // should never occur
        }
    }
    
    pub fn to_expression(&self) -> Box<Expression> {
        match self {
            ConcreteArg::Expr(e) => e.clone(),
            _ => Box::new(Expression::Value(Value::Number(0))), // should never occur
        }
    }
}

impl AbstractEffect {
    pub fn make_concrete(self, context: BaliContext, alt_variables: &mut AltVariableGenerator) -> TopLevelEffect {
        TopLevelEffect::With(vec![Self::internal_make_concrete(self.concrete_type, self.args, Vec::new(), alt_variables)], context)
    } 

    // gérer les arguments un par un
    fn internal_make_concrete(
        effect_type: EffectType, 
        abstract_args: Vec<AbstractArg>, 
        concrete_args: Vec<ConcreteArg>,
        alt_variables: &mut AltVariableGenerator) 
        -> TopLevelEffect {

            if abstract_args.len() == 0 {
                return match effect_type {
                    EffectType::Definition => TopLevelEffect::Effect(Effect::Definition(concrete_args[1].to_value(), concrete_args[0].to_expression()), BaliContext::new()),
                    EffectType::Note => TopLevelEffect::Effect(Effect::Note(concrete_args[0].to_expression(), BaliContext::new()), BaliContext::new()),
                    EffectType::ProgramChange => TopLevelEffect::Effect(Effect::ProgramChange(concrete_args[0].to_expression(), BaliContext::new()), BaliContext::new()),
                    EffectType::ControlChange => TopLevelEffect::Effect(Effect::ControlChange(concrete_args[1].to_expression(), concrete_args[0].to_expression(), BaliContext::new()), BaliContext::new()),
                    EffectType::Aftertouch => TopLevelEffect::Effect(Effect::Aftertouch(concrete_args[1].to_expression(), concrete_args[0].to_expression(), BaliContext::new()), BaliContext::new()),
                    EffectType::ChannelPressure => TopLevelEffect::Effect(Effect::ChannelPressure(concrete_args[0].to_expression(), BaliContext::new()), BaliContext::new()),
                    _ => todo!()
                }
            }

            let mut abstract_args = abstract_args.clone();
            let current_arg = abstract_args.pop().unwrap();

            Self::arg_make_concrete(effect_type, abstract_args, concrete_args, current_arg, alt_variables)
    }

    // descendre dans l'arbre d'un argument
    fn arg_make_concrete(
        effect_type: EffectType,
        abstract_args: Vec<AbstractArg>,
        concrete_args: Vec<ConcreteArg>,
        current_arg: AbstractArg,
        alt_variables: &mut AltVariableGenerator)
        -> TopLevelEffect {

            match current_arg {
                AbstractArg::Alt(args) => {
                    let mut inside = Vec::new();
                    for a in args {
                        inside.push(Self::arg_make_concrete(effect_type.clone(), abstract_args.clone(), concrete_args.clone(), a, alt_variables));
                    } 
                    TopLevelEffect::Alt(inside, alt_variables.get_variable(), BaliContext::new()) // todo: variable (générer)
                },
                AbstractArg::Choice(args) => {
                    let mut inside = Vec::new();
                    for a in args {
                        inside.push(Self::arg_make_concrete(effect_type.clone(), abstract_args.clone(), concrete_args.clone(), a, alt_variables));
                    }
                    TopLevelEffect::Choice(1, inside.len() as i64, inside, BaliContext::new())
                },
                AbstractArg::List(args) => {
                    let mut inside = Vec::new();
                    for a in args {
                        inside.push(Self::arg_make_concrete(effect_type.clone(), abstract_args.clone(), concrete_args.clone(), a, alt_variables));
                    }
                    TopLevelEffect::Seq(inside, BaliContext::new())
                },
                AbstractArg::Concrete(arg) => {
                    let mut concrete_args = concrete_args.clone();
                    concrete_args.push(arg);
                    Self::internal_make_concrete(effect_type, abstract_args, concrete_args, alt_variables)
                },
            }
    }
}