use crate::compiler::bali::bali_ast::{
    BaliContext, TopLevelEffect, Effect, AltVariableGenerator, Expression, Value, Variable
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
    pub dirt_args_names: Vec<String>,
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
        let (effect, _vars) = Self::internal_make_concrete(self.concrete_type, self.args, self.dirt_args_names, Vec::new(), alt_variables, Vec::new());
        TopLevelEffect::With(vec![effect], context)
    } 

    // gérer les arguments un par un
    fn internal_make_concrete(
        effect_type: EffectType, 
        abstract_args: Vec<AbstractArg>, 
        dirt_args_names: Vec<String>,
        concrete_args: Vec<ConcreteArg>,
        alt_variables: &mut AltVariableGenerator,
        alt_variables_set: Vec<Variable>) 
        -> (TopLevelEffect, Vec<Variable>) {

            if abstract_args.len() == 0 {
                let effect = match effect_type {
                    EffectType::Definition => TopLevelEffect::Effect(Effect::Definition(concrete_args[1].to_value(), concrete_args[0].to_expression()), BaliContext::new()),
                    EffectType::Note => TopLevelEffect::Effect(Effect::Note(concrete_args[0].to_expression(), BaliContext::new()), BaliContext::new()),
                    EffectType::ProgramChange => TopLevelEffect::Effect(Effect::ProgramChange(concrete_args[0].to_expression(), BaliContext::new()), BaliContext::new()),
                    EffectType::ControlChange => TopLevelEffect::Effect(Effect::ControlChange(concrete_args[1].to_expression(), concrete_args[0].to_expression(), BaliContext::new()), BaliContext::new()),
                    EffectType::Aftertouch => TopLevelEffect::Effect(Effect::Aftertouch(concrete_args[1].to_expression(), concrete_args[0].to_expression(), BaliContext::new()), BaliContext::new()),
                    EffectType::ChannelPressure => TopLevelEffect::Effect(Effect::ChannelPressure(concrete_args[0].to_expression(), BaliContext::new()), BaliContext::new()),
                    EffectType::Osc => {
                        let mut concrete_args = concrete_args;
                        let addr = concrete_args.pop().unwrap();
                        concrete_args.reverse();
                        TopLevelEffect::Effect(Effect::Osc(addr.to_value(), concrete_args.into_iter().map(|exp_arg| *(exp_arg.to_expression())).collect(), BaliContext::new()), BaliContext::new())
                    },
                    EffectType::Dirt => {
                        let mut concrete_args = concrete_args;
                        let sound = concrete_args.pop().unwrap();
                        concrete_args.reverse();
                        let mut dirt_args = Vec::new();
                        for (pos, arg) in concrete_args.into_iter().enumerate() {
                            dirt_args.push((dirt_args_names[pos].clone(), arg.to_expression()));
                        }
                        TopLevelEffect::Effect(Effect::Dirt(sound.to_value(), dirt_args, BaliContext::new()), BaliContext::new())
                    }
                    //_ => todo!()
                };
                return (effect, Vec::new())
            }

            let mut abstract_args = abstract_args.clone();
            let current_arg = abstract_args.pop().unwrap();

            Self::arg_make_concrete(effect_type, abstract_args, dirt_args_names, concrete_args, current_arg, alt_variables, alt_variables_set.clone())
    }

    // descendre dans l'arbre d'un argument
    fn arg_make_concrete(
        effect_type: EffectType,
        abstract_args: Vec<AbstractArg>,
        dirt_args_names: Vec<String>,
        concrete_args: Vec<ConcreteArg>,
        current_arg: AbstractArg,
        alt_variables: &mut AltVariableGenerator,
        mut alt_variables_set: Vec<Variable>)
        -> (TopLevelEffect, Vec<Variable>) {

            match current_arg {
                AbstractArg::Alt(args) => {
                    let mut inside = Vec::new();
                    let variable = alt_variables_set.pop().unwrap_or(alt_variables.get_variable());
                    for a in args {
                        let (top_level_effect, new_alt_variables_set) = Self::arg_make_concrete(effect_type.clone(), abstract_args.clone(), dirt_args_names.clone(), concrete_args.clone(), a, alt_variables, alt_variables_set.clone());
                        inside.push(top_level_effect);
                        alt_variables_set = new_alt_variables_set;
                    }
                    alt_variables_set.push(variable.clone());
                    (TopLevelEffect::Alt(inside, variable, BaliContext::new()), alt_variables_set)
                },
                AbstractArg::Choice(args) => {
                    let mut inside = Vec::new();
                    for a in args {
                        let (top_level_effect, new_alt_variables_set) = Self::arg_make_concrete(effect_type.clone(), abstract_args.clone(), dirt_args_names.clone(), concrete_args.clone(), a, alt_variables, alt_variables_set.clone());
                        inside.push(top_level_effect);
                        alt_variables_set = new_alt_variables_set;
                    }
                    (TopLevelEffect::Choice(1, inside.len() as i64, inside, BaliContext::new()), alt_variables_set)
                },
                AbstractArg::List(args) => {
                    let mut inside = Vec::new();
                    for a in args {
                        let (top_level_effect, new_alt_variables_set) = Self::arg_make_concrete(effect_type.clone(), abstract_args.clone(), dirt_args_names.clone(), concrete_args.clone(), a, alt_variables, alt_variables_set.clone());
                        inside.push(top_level_effect);
                        alt_variables_set = new_alt_variables_set;
                    }
                    (TopLevelEffect::Seq(inside, BaliContext::new()), alt_variables_set)
                },
                AbstractArg::Concrete(arg) => {
                    let mut concrete_args = concrete_args.clone();
                    concrete_args.push(arg);
                    Self::internal_make_concrete(effect_type, abstract_args, dirt_args_names, concrete_args, alt_variables, alt_variables_set)
                },
            }
    }
}