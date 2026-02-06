use crate::bali::bali_ast::{
    BaliContext, LoopContext, Statement, args::AbstractArg, args::ConcreteArg,
};

#[derive(Debug, Clone)]
pub enum StatementType {
    AfterFrac,
    BeforeFrac,
    Loop,
    Euclidean,
    Binary,
    Choice,
    Spread,
    Pick,
    Ramp,
}

pub struct AbstractStatement {
    pub concrete_type: StatementType,
    pub args: Vec<AbstractArg>,
    pub inside_statements: Vec<Statement>,
    pub loop_context: LoopContext,
}

impl AbstractStatement {
    pub fn make_concrete(self, context: BaliContext) -> Statement {
        let statement = Self::internal_make_concrete(
            self.concrete_type,
            self.args,
            Vec::new(),
            self.inside_statements,
            self.loop_context,
        );
        Statement::With(vec![statement], context)
    }

    // gérer les arguments un par un
    fn internal_make_concrete(
        statement_type: StatementType,
        abstract_args: Vec<AbstractArg>,
        concrete_args: Vec<ConcreteArg>,
        inside_statements: Vec<Statement>,
        loop_context: LoopContext,
    ) -> Statement {
        if abstract_args.is_empty() {
            let statement = match statement_type {
                StatementType::AfterFrac => Statement::AfterFrac(
                    concrete_args[0].to_timing_information(),
                    inside_statements,
                    BaliContext::new(),
                ),
                StatementType::BeforeFrac => Statement::BeforeFrac(
                    concrete_args[0].to_timing_information(),
                    inside_statements,
                    BaliContext::new(),
                ),
                StatementType::Ramp => Statement::Ramp(
                    concrete_args[5].to_value(),
                    concrete_args[4].to_integer(),
                    concrete_args[3].to_integer(),
                    concrete_args[2].to_integer(),
                    concrete_args[1].to_value(),
                    loop_context,
                    concrete_args[0].to_timing_information(),
                    inside_statements,
                    BaliContext::new(),
                ),
                StatementType::Loop => Statement::Loop(
                    concrete_args[1].to_integer(),
                    concrete_args[0].to_timing_information(),
                    inside_statements,
                    loop_context,
                    BaliContext::new(),
                ),
                StatementType::Euclidean => Statement::Euclidean(
                    concrete_args[2].to_integer(),
                    concrete_args[1].to_integer(),
                    loop_context,
                    concrete_args[0].to_timing_information(),
                    inside_statements,
                    BaliContext::new(),
                ),
                StatementType::Choice => Statement::Choice(
                    concrete_args[0].to_integer(),
                    inside_statements.len() as i64,
                    inside_statements,
                    BaliContext::new(),
                ),
                StatementType::Binary => Statement::Binary(
                    concrete_args[2].to_integer(),
                    concrete_args[1].to_integer(),
                    loop_context,
                    concrete_args[0].to_timing_information(),
                    inside_statements,
                    BaliContext::new(),
                ),
                StatementType::Spread => Statement::Spread(
                    concrete_args[0].to_timing_information(),
                    inside_statements,
                    loop_context,
                    BaliContext::new(),
                ),
                StatementType::Pick => Statement::Pick(
                    concrete_args[0].to_expression(),
                    inside_statements,
                    BaliContext::new(),
                ),
            };
            return statement;
        }

        let mut abstract_args = abstract_args.clone();
        let current_arg = abstract_args.pop().unwrap();

        Self::arg_make_concrete(
            statement_type,
            abstract_args,
            concrete_args,
            current_arg,
            inside_statements,
            loop_context,
        )
    }

    // descendre dans l'arbre d'un argument
    fn arg_make_concrete(
        statement_type: StatementType,
        abstract_args: Vec<AbstractArg>,
        concrete_args: Vec<ConcreteArg>,
        current_arg: AbstractArg,
        inside_statements: Vec<Statement>,
        loop_context: LoopContext,
    ) -> Statement {
        match current_arg {
            AbstractArg::Alt(args, variable) => {
                let mut inside = Vec::new();
                for a in args {
                    let statement = Self::arg_make_concrete(
                        statement_type.clone(),
                        abstract_args.clone(),
                        concrete_args.clone(),
                        a,
                        inside_statements.clone(),
                        loop_context.clone(),
                    );
                    inside.push(statement);
                }
                Statement::Alt(inside, variable, BaliContext::new())
            }
            AbstractArg::Choice(args) => {
                let mut inside = Vec::new();
                for a in args {
                    let statement = Self::arg_make_concrete(
                        statement_type.clone(),
                        abstract_args.clone(),
                        concrete_args.clone(),
                        a,
                        inside_statements.clone(),
                        loop_context.clone(),
                    );
                    inside.push(statement);
                }
                Statement::Choice(1, inside.len() as i64, inside, BaliContext::new())
            }
            AbstractArg::List(args) => {
                let mut inside = Vec::new();
                for a in args {
                    let statement = Self::arg_make_concrete(
                        statement_type.clone(),
                        abstract_args.clone(),
                        concrete_args.clone(),
                        a,
                        inside_statements.clone(),
                        loop_context.clone(),
                    );
                    inside.push(statement);
                }
                Statement::With(inside, BaliContext::new())
            }
            AbstractArg::Concrete(arg) => {
                let mut concrete_args = concrete_args.clone();
                concrete_args.push(arg);
                Self::internal_make_concrete(
                    statement_type,
                    abstract_args,
                    concrete_args,
                    inside_statements,
                    loop_context,
                )
            }
        }
    }
}
