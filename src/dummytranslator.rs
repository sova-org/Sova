use lalrpop_util::lalrpop_mod;
use crate::lang::Program;

lalrpop_mod!(pub dummygrammar);

pub fn translate(script: &str) -> Program {
    dummygrammar::ProgParser::new().parse(script).unwrap().as_asm()
}