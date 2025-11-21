use pest::{Parser, iterators::Pairs, pratt_parser::PrattParser};
use pest_derive::Parser;

use crate::{clock::{SyncTime, TimeSpan}, compiler::CompilationError, lang::interpreter::boinx::ast::{BoinxArithmeticOp, BoinxCompo, BoinxCompoOp, BoinxCondition, BoinxConditionOp, BoinxIdent, BoinxIdentQualif, BoinxItem, BoinxOutput, BoinxProg, BoinxStatement}};

#[derive(Parser)]
#[grammar = "lang/interpreter/boinx/boinx.pest"]
pub struct BoinxParser;

lazy_static::lazy_static! {
    static ref BOINX_PRATT_PARSER : PrattParser<Rule> = {
        use pest::pratt_parser::{Assoc::*, Op};
        use Rule::*;
        // Precedence is defined lowest to highest
        PrattParser::new()
            .op(
                Op::infix(compo_op, Right) | 
                Op::infix(iter_op, Right) |
                Op::infix(each_op, Right) 
            )
            .op(
                Op::infix(shr, Left) | 
                Op::infix(shl, Left) 
            )
            .op(
                Op::infix(add, Left) | 
                Op::infix(sub, Left) | 
                Op::infix(rem, Left)
            )
            .op(
                Op::infix(mul, Left) | 
                Op::infix(div, Left)
            )
            .op(Op::infix(pow, Left))
            .op(Op::prefix(minus))
    };
}

fn parse_ident(pairs: Pairs<Rule>) -> BoinxIdent {
    let mut name = String::new();
    let mut qualif = BoinxIdentQualif::default();
    for pair in pairs {
        match pair.as_rule() {
            Rule::name => name = pair.as_str().to_owned(),
            Rule::env_func => qualif = BoinxIdentQualif::EnvFunc,
            Rule::seq_var => qualif = BoinxIdentQualif::SeqVar,
            _ => unreachable!()
        }
    }
    BoinxIdent(name, qualif)
}

fn parse_note(pairs: Pairs<Rule>) -> i64 {
    let mut i = 0;
    let mut has_octave = false;
    for pair in pairs {
        match pair.as_rule() {
            Rule::note_letter => i = match pair.as_str() {
                "C" => 0,
                "D" => 2,
                "E" => 4,
                "F" => 5,
                "G" => 7,
                "A" => 9,
                "B" => 11,
                _ => unreachable!()
            },
            Rule::sharp => i += 1,
            Rule::flat => i -= 1,
            Rule::int => {
                has_octave = true;
                i += pair.as_str().parse::<i64>().unwrap_or_default() * 12;
            },
            _ => unreachable!()
        }
    }
    if !has_octave {
        i += 12 * 4;
    }
    i
}

fn parse_condition(mut pairs: Pairs<Rule>) -> BoinxCondition {
    let lhs = pairs.next().unwrap();
    let lhs = parse_compo(lhs.into_inner()).extract();
    let op = match pairs.next().unwrap().as_rule() {
        Rule::lt => BoinxConditionOp::Less,
        Rule::le => BoinxConditionOp::LessEq,
        Rule::eq => BoinxConditionOp::Equal,
        Rule::ge => BoinxConditionOp::Greater,
        Rule::gt => BoinxConditionOp::GreaterEq,
        Rule::neq => BoinxConditionOp::NotEqual,
        _ => unreachable!()
    };
    let rhs = pairs.next().unwrap();
    let rhs = parse_compo(rhs.into_inner()).extract();
    BoinxCondition(Box::new(lhs), op, Box::new(rhs))
}

fn parse_compo(pairs: Pairs<Rule>) -> BoinxCompo {
    BOINX_PRATT_PARSER
        .map_primary(|primary| match primary.as_rule() {
            Rule::int => {
                let i = primary.as_str().parse().unwrap_or_default();
                BoinxItem::Note(i).into()
            }
            Rule::prev => BoinxItem::Previous.into(),
            Rule::stop => BoinxItem::Stop.into(),
            Rule::note => 
                BoinxItem::Note(parse_note(primary.into_inner())).into(),
            Rule::real => {
                let f = primary.as_str().parse().unwrap_or_default();
                BoinxItem::Number(f).into()
            }
            Rule::ident => 
                BoinxItem::Identity(parse_ident(primary.into_inner())).into(),
            Rule::micros => {
                let mut inner = primary.into_inner();
                let inner = inner.next().unwrap();
                let date = inner.as_str().parse::<SyncTime>().unwrap_or_default();
                BoinxItem::Duration(TimeSpan::Micros(date)).into()
            }
            Rule::semibeats => {
                let mut inner = primary.into_inner();
                let inner = inner.next().unwrap();
                let ts = inner.as_str().parse::<f64>().unwrap_or_default();
                BoinxItem::Duration(TimeSpan::Beats(ts / 2.0)).into()
            }
            Rule::beats => {
                let mut inner = primary.into_inner();
                let inner = inner.next().unwrap();
                let ts = inner.as_str().parse::<f64>().unwrap_or_default();
                BoinxItem::Duration(TimeSpan::Beats(ts)).into()
            }
            Rule::mute => BoinxItem::Mute.into(),
            Rule::placeholder => BoinxItem::Placeholder.into(),
            Rule::sub_prog => {
                let prog = parse_prog(primary.into_inner());
                BoinxItem::SubProg(Box::new(prog)).into()
            },
            Rule::if_else => {
                let mut inner = primary.into_inner();
                let condition = inner.next().unwrap().into_inner();
                let t_block = inner.next().unwrap().into_inner();
                let f_block = inner.next().unwrap().into_inner();
                BoinxItem::Condition(
                    parse_condition(condition), 
                    Box::new(parse_prog(t_block)), 
                    Box::new(parse_prog(f_block))
                ).into()
            },
            Rule::seque => {
                let vec = primary.into_inner()
                    .map(|p| parse_compo(p.into_inner()).extract())
                    .collect();
                BoinxItem::Sequence(vec).into()
            }
            Rule::simul => {
                let vec = primary.into_inner()
                    .map(|p| parse_compo(p.into_inner()).extract())
                    .collect();
                BoinxItem::Simultaneous(vec).into()
            }
            _ => unreachable!()
        })
        .map_infix(|lhs: BoinxCompo, op, rhs: BoinxCompo| match op.as_rule() {
            Rule::compo_op => lhs.chain(BoinxCompoOp::Compose, rhs),
            Rule::iter_op => lhs.chain(BoinxCompoOp::Iterate, rhs),
            Rule::each_op => lhs.chain(BoinxCompoOp::Each, rhs),
            _ => {
                let op = match op.as_rule() {
                    Rule::add => BoinxArithmeticOp::Add,
                    Rule::sub => BoinxArithmeticOp::Sub,
                    Rule::mul => BoinxArithmeticOp::Mul,
                    Rule::div => BoinxArithmeticOp::Div,
                    Rule::rem => BoinxArithmeticOp::Rem,
                    Rule::shl => BoinxArithmeticOp::Shl,
                    Rule::shr => BoinxArithmeticOp::Shr,
                    Rule::pow => BoinxArithmeticOp::Pow,
                    _ => unreachable!()
                };
                BoinxItem::Arithmetic(
                    Box::new(lhs.extract()), 
                    op,
                    Box::new(rhs.extract())
                ).into()
            }
        })
        .map_prefix(|op, rhs| match op.as_rule() {
            Rule::minus => 
                BoinxItem::Negative(Box::new(rhs.extract())).into(),
            _ => unreachable!()
        })
        .parse(pairs)
}

fn parse_prog(pairs: Pairs<Rule>) -> BoinxProg {
    let mut statements = Vec::new();
    for pair in pairs {
        match pair.as_rule() {
            Rule::output => {
                let inner = pair.into_inner();
                let mut compo = BoinxCompo::default();
                let mut device = None;
                let mut channel = None;
                for in_pair in inner {
                    match in_pair.as_rule() {
                        Rule::compo => compo = parse_compo(in_pair.into_inner()),
                        Rule::dev => 
                            device = Some(parse_compo(in_pair.into_inner()).extract()),
                        Rule::chan => 
                            channel = Some(parse_compo(in_pair.into_inner()).extract()),
                        _ => unreachable!()
                    }
                }
                let output = BoinxOutput {
                    compo, device, channel
                };
                statements.push(BoinxStatement::Output(output));
            }
            Rule::assign => {
                let mut inner = pair.into_inner();
                let ident = parse_ident(inner.next().unwrap().into_inner());
                let compo = parse_compo(inner.next().unwrap().into_inner());
                let assign = BoinxStatement::Assign(ident, compo);
                statements.push(assign);
            }
            Rule::EOI => break,
            rule => unreachable!("Unreachable expression: {rule:?}")
        }
    }
    BoinxProg(statements)
}

pub fn parse_boinx(prog: &str) -> Result<BoinxProg, CompilationError> {
    match BoinxParser::parse(Rule::prog, prog) {
        Ok(pairs) => {
            Ok(parse_prog(pairs))
        },
        Err(e) => Err(CompilationError { 
            lang: "boinx".to_owned(), 
            info: format!("Parsing error: {e}"), 
            from: 0, 
            to: 0 
        }),
    }
}