use crate::compiler::bali::bali_ast::{
    AltVariableGenerator, ChoiceVariableGenerator, ConcreteFraction, LocalChoiceVariableGenerator,
    bali_context::BaliContext,
    expression::Expression,
    function::FunctionContent,
    information::{
        AltInformation, ChoiceInformation, Information, PickInformation, RampInformation,
        TimingInformation,
    },
    loop_context::LoopContext,
    time_statement::TimeStatement,
    toplevel_effect::TopLevelEffect,
    value::Value,
};
use crate::lang::variable::Variable;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Statement {
    AfterFrac(TimingInformation, Vec<Statement>, BaliContext),
    BeforeFrac(TimingInformation, Vec<Statement>, BaliContext),
    Loop(
        i64,
        TimingInformation,
        Vec<Statement>,
        LoopContext,
        BaliContext,
    ),
    Euclidean(
        i64,
        i64,
        LoopContext,
        TimingInformation,
        Vec<Statement>,
        BaliContext,
    ),
    Binary(
        i64,
        i64,
        LoopContext,
        TimingInformation,
        Vec<Statement>,
        BaliContext,
    ),
    After(Vec<TopLevelEffect>, BaliContext),
    Before(Vec<TopLevelEffect>, BaliContext),
    Effect(TopLevelEffect),
    With(Vec<Statement>, BaliContext),
    Choice(i64, i64, Vec<Statement>, BaliContext), // Choice(num, tot, ss, c) num chances sur tot de faire chaque chose de ss (si tot = ss.len() on en fait exactement num parmi les ss, si tot > ss.len() on en fait num parmi un vecteur dont le début et ss et les éléments suivants sont vides qui est de taille tot)
    Spread(TimingInformation, Vec<Statement>, LoopContext, BaliContext), // Spread(timeStep, ss, c) effectue les statements de ss en les séparant d'un temps timeStep (la première à 0, la deuxième à timeStep, la troisième à 2*timeStep, etc)
    Pick(Box<Expression>, Vec<Statement>, BaliContext), // sélectionne le Statement dont le numéro est indiqué par la valeur de l'expression (modulo le nombre de Statements), l'expression est évaluée au moment du Statement qui arrive le plus tôt
    Alt(Vec<Statement>, Variable, BaliContext), // Sélectionne un statement différent (dans l'ordre) à chaque fois qu'on passe
    Ramp(
        Value,
        i64,
        i64,
        i64,
        Value,
        LoopContext,
        TimingInformation,
        Vec<Statement>,
        BaliContext,
    ),
    FunctionDeclaration(Value, Vec<Value>, Vec<TopLevelEffect>, Box<Expression>),
}

impl Statement {
    /*
    pub fn set_context(self, c: BaliContext) -> Statement {
        match self {
            Statement::AfterFrac(v, es, cc) => Statement::AfterFrac(v, es, cc.update(c)),
            Statement::BeforeFrac(v, es, cc) => Statement::BeforeFrac(v, es, cc.update(c)),
            Statement::Loop(it, v, es, cc) => Statement::Loop(it, v, es, cc.update(c)),
            Statement::After(es, cc) => Statement::After(es, cc.update(c)),
            Statement::Before(es, cc) => Statement::Before(es, cc.update(c)),
            Statement::Effect(e, cc) => Statement::Effect(e, cc.update(c)),
        }
    }
    */

    fn is_simplifiable(seq: &Vec<Vec<i64>>) -> bool {
        if seq.len() < 2 {
            return false;
        }

        return seq[seq.len() - 1].len() == seq[seq.len() - 2].len()
            && seq[seq.len() - 1].len() != seq[0].len();
    }

    fn get_euclidean(beats: i64, steps: i64, context: LoopContext) -> Vec<i64> {
        let mut seqs: Vec<Vec<i64>> = Vec::new();

        for _i in 0..beats {
            seqs.push(vec![1]);
        }

        let seqs_len = seqs.len();
        for j in 0..(steps - beats) {
            seqs[j as usize % seqs_len].push(0);
        }

        while Self::is_simplifiable(&seqs) {
            let mut in_pos = seqs.len() - 1;
            let mut out_pos = 0;
            let last = seqs[in_pos].len();
            while seqs[in_pos].len() == last {
                if let Some(elem) = seqs.pop() {
                    seqs[out_pos].extend(elem);
                    in_pos -= 1;
                    out_pos += 1;
                    if out_pos >= seqs.len() || seqs[out_pos].len() == last {
                        out_pos = 0;
                    }
                }
            }
        }

        let mut seq: Vec<i64> = seqs.into_iter().flatten().collect();

        Self::as_time_points(&mut seq, context)
    }

    fn get_binary(it: i64, steps: i64, context: LoopContext) -> Vec<i64> {
        let mut seq = Vec::new();
        let mut bin_seq = it;

        for _i in 0..8 {
            seq.push(bin_seq % 2);
            bin_seq = bin_seq / 2;
        }
        seq.reverse();

        let mut res_seq = Vec::new();
        for i in 0..steps {
            res_seq.push(seq[(i % 8) as usize]);
        }

        Self::as_time_points(&mut res_seq, context)
    }

    fn as_time_points(seq: &mut Vec<i64>, context: LoopContext) -> Vec<i64> {
        //print!("{:?}\n", seq);

        if context.reverse {
            seq.reverse();
        }

        if context.negate {
            seq.iter_mut().for_each(|x| *x = 1 - *x);
        }

        if let Some(shift) = context.shift {
            seq.rotate_right(shift as usize);
        }

        //print!("{:?}\n", seq);

        let mut res = Vec::new();
        let mut count = 0;
        for i in 0..seq.len() {
            if seq[i] == 1 {
                res.push(count);
            }
            count += 1;
        }

        //print!("{:?}\n", res);

        res
    }

    fn get_linear_distribution(start: i64, end: i64, steps: i64) -> Vec<i64> {
        let mut res = Vec::new();

        let coeff = ((end - start) as f64) / ((steps - 1) as f64);

        for x in 0..steps {
            let y = coeff * (x as f64) + (start as f64);
            res.push(y as i64);
        }

        res
    }

    pub fn get_function(
        &self,
        functions_map: &mut HashMap<String, FunctionContent>,
    ) -> Result<(), String> {
        match self {
            Statement::FunctionDeclaration(
                func_name,
                func_args,
                toplevel_effects,
                return_expression,
            ) => {
                let key = func_name.to_str();
                if functions_map.contains_key(&key) {
                    return Err(format!("Duplicate definition of function {}", key));
                }

                let mut check_args: Vec<String> =
                    func_args.into_iter().map(|arg| arg.to_str()).collect();
                let num_args = check_args.len();
                check_args.sort();
                check_args.dedup();
                if check_args.len() != num_args {
                    return Err(format!("Duplicate argument names in function {}", key));
                }

                functions_map.insert(
                    key,
                    FunctionContent {
                        arg_list: func_args.into_iter().map(|arg| arg.to_str()).collect(),
                        return_expression: return_expression.clone(),
                        function_program: toplevel_effects.clone(),
                    },
                );
                Ok(())
            }
            _ => Ok(()),
        }
    }

    pub fn expend(
        self,
        val: &ConcreteFraction,
        spread_time: &ConcreteFraction,
        c: BaliContext,
        infos: Vec<Information>,
        choice_vars: &mut ChoiceVariableGenerator,
        pick_vars: &mut LocalChoiceVariableGenerator,
        alt_vars: &mut AltVariableGenerator,
    ) -> Vec<TimeStatement> {
        /*let c = match self {
            Statement::AfterFrac(_, _, ref cc) | Statement::BeforeFrac(_, _, ref cc) | Statement::Loop(_, _, _, ref cc) | Statement::After(_, ref cc) | Statement::Before(_, ref cc) | Statement::Effect(_, ref cc) => cc.clone().update(c),
        };*/
        match self {
            Statement::FunctionDeclaration(
                _func_name,
                _func_args,
                _toplevel_effects,
                _return_expression,
            ) => {
                // function declarations do not generate any TimeStatement
                Vec::new()
            }
            Statement::AfterFrac(v, es, cc) => es
                .into_iter()
                .map(|e| {
                    e.expend(
                        &v.as_frames(spread_time).add(val),
                        spread_time,
                        cc.clone().update(c.clone()),
                        infos.clone(),
                        choice_vars,
                        pick_vars,
                        alt_vars,
                    )
                })
                .flatten()
                .collect(),
            Statement::BeforeFrac(v, es, cc) => es
                .into_iter()
                .map(|e| {
                    e.expend(
                        &val.sub(&v.as_frames(spread_time)),
                        spread_time,
                        cc.clone().update(c.clone()),
                        infos.clone(),
                        choice_vars,
                        pick_vars,
                        alt_vars,
                    )
                })
                .flatten()
                .collect(),
            Statement::Loop(it, v, es, loop_context, cc) => {
                let mut res = Vec::new();
                let mut v = v.as_frames(spread_time);
                if !loop_context.step_time {
                    v = v.divbyint(it);
                }
                for i in 0..it {
                    let content: Vec<TimeStatement> = es
                        .clone()
                        .into_iter()
                        .map(|e| {
                            e.expend(
                                &val.add(&v.multbyint(i)),
                                &v,
                                cc.clone().update(c.clone()),
                                infos.clone(),
                                choice_vars,
                                pick_vars,
                                alt_vars,
                            )
                        })
                        .flatten()
                        .collect();
                    res.extend(content);
                }
                res
            }
            Statement::Ramp(
                var,
                granularity,
                start,
                end,
                distribution,
                loop_context,
                v,
                es,
                cc,
            ) => {
                let mut res = Vec::new();

                let granularity = if granularity < 2 { 2 } else { granularity };

                let mut v = v.as_frames(spread_time);
                if !loop_context.step_time {
                    v = v.divbyint(granularity);
                }

                let ramp_values = match distribution.to_str().as_str() {
                    "linear" => Self::get_linear_distribution(start, end, granularity),
                    "exponential" => todo!(),
                    _ => Self::get_linear_distribution(start, end, granularity),
                };

                if let Value::Variable(var) = var {
                    for i in 0..granularity {
                        let new_ramp_info = RampInformation {
                            variable_name: var.clone(),
                            variable_value: ramp_values[i as usize],
                        };
                        let mut new_infos = vec![Information::Ramp(new_ramp_info)];
                        new_infos.extend(infos.clone());
                        let content: Vec<TimeStatement> = es
                            .clone()
                            .into_iter()
                            .map(|e| {
                                e.expend(
                                    &val.add(&v.multbyint(i)),
                                    &v,
                                    cc.clone().update(c.clone()),
                                    new_infos.clone(),
                                    choice_vars,
                                    pick_vars,
                                    alt_vars,
                                )
                            })
                            .flatten()
                            .collect();
                        res.extend(content);
                    }
                }
                res
            }
            Statement::Euclidean(beats, steps, loop_context, v, es, cc) => {
                let mut res = Vec::new();
                let mut v = v.as_frames(spread_time);
                if !loop_context.step_time {
                    v = v.divbyint(steps);
                }
                let euc = Self::get_euclidean(beats, steps, loop_context);
                for i in 0..euc.len() {
                    let content: Vec<TimeStatement> = es
                        .clone()
                        .into_iter()
                        .map(|e| {
                            e.expend(
                                &val.add(&v.multbyint(euc[i])),
                                &v,
                                cc.clone().update(c.clone()),
                                infos.clone(),
                                choice_vars,
                                pick_vars,
                                alt_vars,
                            )
                        })
                        .flatten()
                        .collect();
                    res.extend(content);
                }
                res
            }
            Statement::Binary(it, steps, loop_context, v, es, cc) => {
                let mut res = Vec::new();
                let mut v = v.as_frames(spread_time);
                if !loop_context.step_time {
                    v = v.divbyint(steps);
                }
                let bin = Self::get_binary(it, steps, loop_context);
                for i in 0..bin.len() {
                    let content: Vec<TimeStatement> = es
                        .clone()
                        .into_iter()
                        .map(|e| {
                            e.expend(
                                &val.add(&v.multbyint(bin[i])),
                                &v,
                                cc.clone().update(c.clone()),
                                infos.clone(),
                                choice_vars,
                                pick_vars,
                                alt_vars,
                            )
                        })
                        .flatten()
                        .collect();
                    res.extend(content);
                }
                res
            }
            Statement::After(es, cc) => es
                .into_iter()
                .map(|e| {
                    TimeStatement::JustAfter(
                        val.clone(),
                        e,
                        cc.clone().update(c.clone()),
                        infos.clone(),
                    )
                })
                .collect(),
            Statement::Before(es, cc) => es
                .into_iter()
                .map(|e| {
                    TimeStatement::JustBefore(
                        val.clone(),
                        e,
                        cc.clone().update(c.clone()),
                        infos.clone(),
                    )
                })
                .collect(),
            Statement::Effect(e) => {
                vec![TimeStatement::At(val.clone(), e, c.clone(), infos.clone())]
            }
            Statement::With(es, cc) => es
                .into_iter()
                .map(|e| {
                    e.expend(
                        val,
                        spread_time,
                        cc.clone().update(c.clone()),
                        infos.clone(),
                        choice_vars,
                        pick_vars,
                        alt_vars,
                    )
                })
                .flatten()
                .collect(),
            Statement::Choice(num_selected, num_selectable, es, cc) => {
                let mut res = Vec::new();

                if num_selected == 0 {
                    return res;
                }

                if num_selected >= num_selectable {
                    return es
                        .into_iter()
                        .map(|e| {
                            e.expend(
                                val,
                                spread_time,
                                cc.clone().update(c.clone()),
                                infos.clone(),
                                choice_vars,
                                pick_vars,
                                alt_vars,
                            )
                        })
                        .flatten()
                        .collect();
                }

                let (choice_variables, target_variables) =
                    choice_vars.get_variables(num_selected, num_selectable);
                for position in 0..es.len() {
                    let new_choice = ChoiceInformation {
                        variables: choice_variables.clone(),
                        target_variables: target_variables.clone(),
                        //num_selectable,
                        position,
                    };
                    let mut new_infos = vec![Information::Choice(new_choice)];
                    new_infos.extend(infos.clone());
                    //infos.push(Information::Choice(new_choice));
                    res.extend(es[position].clone().expend(
                        val,
                        spread_time,
                        cc.clone().update(c.clone()),
                        new_infos,
                        choice_vars,
                        pick_vars,
                        alt_vars,
                    ));
                }
                res
            }
            Statement::Spread(step, es, loop_context, cc) => {
                let mut res = Vec::new();
                let mut step = step.as_frames(spread_time);
                if !loop_context.step_time {
                    step = step.divbyint(es.len() as i64);
                }
                for i in 0..es.len() {
                    let content: Vec<TimeStatement> = es[i].clone().expend(
                        &val.add(&step.multbyint(i as i64)),
                        &step,
                        cc.clone().update(c.clone()),
                        infos.clone(),
                        choice_vars,
                        pick_vars,
                        alt_vars,
                    );
                    res.extend(content);
                }
                res
            }
            Statement::Pick(pick_expression, es, cc) => {
                let mut res = Vec::new();
                let (pick_variable, num_pick_variable) = pick_vars.get_variable_and_number();
                for position in 0..es.len() {
                    let new_pick = PickInformation {
                        variable: pick_variable.clone(),
                        position,
                        possibilities: es.len(),
                        expression: *pick_expression.clone(),
                        num_variable: num_pick_variable,
                    };
                    let mut new_infos = vec![Information::Pick(new_pick)];
                    new_infos.extend(infos.clone());
                    //infos.push(Information::Pick(new_pick));
                    res.extend(es[position].clone().expend(
                        val,
                        spread_time,
                        cc.clone().update(c.clone()),
                        new_infos,
                        choice_vars,
                        pick_vars,
                        alt_vars,
                    ));
                }
                res
            }
            Statement::Alt(es, frame_variable, cc) => {
                let mut res = Vec::new();
                let (_, instance_variable, num_variable) = alt_vars.get_variables_and_num();
                for position in 0..es.len() {
                    let new_alt = AltInformation {
                        frame_variable: frame_variable.clone(),
                        instance_variable: instance_variable.clone(),
                        position,
                        possibilities: es.len(),
                        num_variable,
                    };
                    let mut new_infos = vec![Information::Alt(new_alt)];
                    new_infos.extend(infos.clone());
                    //infos.push(Information::Alt(new_alt));
                    res.extend(es[position].clone().expend(
                        val,
                        spread_time,
                        cc.clone().update(c.clone()),
                        new_infos,
                        choice_vars,
                        pick_vars,
                        alt_vars,
                    ))
                }

                res
            }
        }
    }
}
