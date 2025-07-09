use crate::compiler::bali::bali_ast::{
    BaliContext, TopLevelEffect, Effect, args::AbstractArg, args::ConcreteArg,
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
    For,
    If,
    Pick,
    Choice,
}

pub struct AbstractEffect {
    pub concrete_type: EffectType,
    pub dirt_args_names: Vec<String>,
    pub args: Vec<AbstractArg>,
    pub inside_effects: Vec<TopLevelEffect>,
}

impl AbstractEffect {
    pub fn make_concrete(self, context: BaliContext) -> TopLevelEffect {
        let effect = Self::internal_make_concrete(self.concrete_type, self.args, self.dirt_args_names, Vec::new(), self.inside_effects);
        TopLevelEffect::With(vec![effect], context)
    } 

    // gérer les arguments un par un
    fn internal_make_concrete(
        effect_type: EffectType, 
        abstract_args: Vec<AbstractArg>, 
        dirt_args_names: Vec<String>,
        concrete_args: Vec<ConcreteArg>,
        inside_effects: Vec<TopLevelEffect>) 
        -> TopLevelEffect {

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
                    },
                    EffectType::For => TopLevelEffect::For(concrete_args[0].to_boolean_expression(), inside_effects, BaliContext::new()),
                    EffectType::If => TopLevelEffect::If(concrete_args[0].to_boolean_expression(), inside_effects, BaliContext::new()),
                    EffectType::Pick => TopLevelEffect::Pick(concrete_args[0].to_expression(), inside_effects, BaliContext::new()),
                    EffectType::Choice => TopLevelEffect::Choice(concrete_args[0].to_integer(), inside_effects.len() as i64, inside_effects, BaliContext::new()),
                    //_ => todo!()
                };
                return effect
            }

            let mut abstract_args = abstract_args.clone();
            let current_arg = abstract_args.pop().unwrap();

            Self::arg_make_concrete(effect_type, abstract_args, dirt_args_names, concrete_args, current_arg, inside_effects)
    }

    // descendre dans l'arbre d'un argument
    fn arg_make_concrete(
        effect_type: EffectType,
        abstract_args: Vec<AbstractArg>,
        dirt_args_names: Vec<String>,
        concrete_args: Vec<ConcreteArg>,
        current_arg: AbstractArg,
        inside_effects: Vec<TopLevelEffect>)
        -> TopLevelEffect {

            match current_arg {
                AbstractArg::Alt(args, variable) => {
                    let mut inside = Vec::new();
                    for a in args {
                        let top_level_effect = Self::arg_make_concrete(effect_type.clone(), abstract_args.clone(), dirt_args_names.clone(), concrete_args.clone(), a, inside_effects.clone());
                        inside.push(top_level_effect);
                    }
                    TopLevelEffect::Alt(inside, variable, BaliContext::new())
                },
                AbstractArg::Choice(args) => {
                    let mut inside = Vec::new();
                    for a in args {
                        let top_level_effect = Self::arg_make_concrete(effect_type.clone(), abstract_args.clone(), dirt_args_names.clone(), concrete_args.clone(), a, inside_effects.clone());
                        inside.push(top_level_effect);
                    }
                    TopLevelEffect::Choice(1, inside.len() as i64, inside, BaliContext::new())
                },
                AbstractArg::List(args) => {
                    let mut inside = Vec::new();
                    for a in args {
                        let top_level_effect = Self::arg_make_concrete(effect_type.clone(), abstract_args.clone(), dirt_args_names.clone(), concrete_args.clone(), a, inside_effects.clone());
                        inside.push(top_level_effect);
                    }
                    TopLevelEffect::Seq(inside, BaliContext::new())
                },
                AbstractArg::Concrete(arg) => {
                    let mut concrete_args = concrete_args.clone();
                    concrete_args.push(arg);
                    Self::internal_make_concrete(effect_type, abstract_args, dirt_args_names, concrete_args, inside_effects)
                },
            }
    }
}