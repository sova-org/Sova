use std::{sync::Arc, collections::HashMap};

use crate::clock::ClockServer;

use super::*;

// TODO: tests pour les durées

// No jump in arithmetic operations
#[test]
fn arithmetic_no_jump() {

    let var_name = "z";

    let inst = ControlASM::Add(
        Variable::Constant(VariableValue::Integer(1)),
        Variable::Constant(VariableValue::Integer(1)),
        Variable::Instance(var_name.to_owned()));

    let clock_server = Arc::new(ClockServer::new(120.0, 4.0));
    let clock = clock_server.into();

    let instance_vars = &mut HashMap::new();

    let index = inst.execute(
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        instance_vars,
        &clock,
    );

    match index {
        None => return,
        Some(_) => panic!("Failed"),
    };
}

// Type casting in arithmetic operations
#[test]
fn arithmetic_cast_to_x() {

    let var_name = "z";

    let inst = ControlASM::Add(
        Variable::Constant(VariableValue::Integer(1)),
        Variable::Constant(VariableValue::Float(1.0)),
        Variable::Instance(var_name.to_owned()));

    let clock_server = Arc::new(ClockServer::new(120.0, 4.0));
    let clock = clock_server.into();

    let instance_vars = &mut HashMap::new();

    inst.execute(
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        instance_vars,
        &clock,
    );

    match instance_vars.get(var_name) {
        Some(VariableValue::Integer(_)) => return,
        _ => panic!("Failed"),
    };
}


#[test]
fn arithmetic_cast_to_y() {

    let var_name = "z";

    let inst = ControlASM::Add(
        Variable::Constant(VariableValue::Bool(true)),
        Variable::Constant(VariableValue::Float(3.0)),
        Variable::Instance(var_name.to_owned()));

    let clock_server = Arc::new(ClockServer::new(120.0, 4.0));
    let clock = clock_server.into();

    let instance_vars = &mut HashMap::new();

    inst.execute(
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        instance_vars,
        &clock,
    );

    match instance_vars.get(var_name) {
        Some(VariableValue::Float(_)) => return,
        _ => panic!("Failed"),
    };
}

#[test]
fn arithmetic_cast_to_int() {

    let var_name = "z";

    let inst = ControlASM::Add(
        Variable::Constant(VariableValue::Bool(true)),
        Variable::Constant(VariableValue::Str("test".to_string())),
        Variable::Instance(var_name.to_owned()));

    let clock_server = Arc::new(ClockServer::new(120.0, 4.0));
    let clock = clock_server.into();

    let instance_vars = &mut HashMap::new();

    inst.execute(
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        instance_vars,
        &clock,
    );

    match instance_vars.get(var_name) {
        Some(VariableValue::Integer(_)) => return,
        _ => panic!("Failed"),
    };
}

// Result of arithmetic operations
#[test]
fn add_float() {

    let var_name = "z";

    let inst = ControlASM::Add(
        Variable::Constant(VariableValue::Float(5.0)),
        Variable::Constant(VariableValue::Float(3.0)),
        Variable::Instance(var_name.to_owned()));

    let clock_server = Arc::new(ClockServer::new(120.0, 4.0));
    let clock = clock_server.into();

    let instance_vars = &mut HashMap::new();

    inst.execute(
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        instance_vars,
        &clock,
    );

    match instance_vars.get(var_name) {
        Some(VariableValue::Float(8.0)) => return,
        _ => panic!("Failed"),
    };
}

#[test]
fn add_integer() {

    let var_name = "z";

    let inst = ControlASM::Add(
        Variable::Constant(VariableValue::Integer(5)),
        Variable::Constant(VariableValue::Integer(3)),
        Variable::Instance(var_name.to_owned()));

    let clock_server = Arc::new(ClockServer::new(120.0, 4.0));
    let clock = clock_server.into();

    let instance_vars = &mut HashMap::new();

    inst.execute(
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        instance_vars,
        &clock,
    );

    match instance_vars.get(var_name) {
        Some(VariableValue::Integer(8)) => return,
        _ => panic!("Failed"),
    };
}

#[test]
fn div_float() {

    let var_name = "z";

    let inst = ControlASM::Div(
        Variable::Constant(VariableValue::Float(6.0)),
        Variable::Constant(VariableValue::Float(2.0)),
        Variable::Instance(var_name.to_owned()));

    let clock_server = Arc::new(ClockServer::new(120.0, 4.0));
    let clock = clock_server.into();

    let instance_vars = &mut HashMap::new();

    inst.execute(
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        instance_vars,
        &clock,
    );

    match instance_vars.get(var_name) {
        Some(VariableValue::Float(3.0)) => return,
        _ => panic!("Failed"),
    };
}

#[test]
fn div_float_zero() {

    let var_name = "z";

    let inst = ControlASM::Div(
        Variable::Constant(VariableValue::Float(6.0)),
        Variable::Constant(VariableValue::Float(0.0)),
        Variable::Instance(var_name.to_owned()));

    let clock_server = Arc::new(ClockServer::new(120.0, 4.0));
    let clock = clock_server.into();

    let instance_vars = &mut HashMap::new();

    inst.execute(
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        instance_vars,
        &clock,
    );

    match instance_vars.get(var_name) {
        Some(VariableValue::Float(0.0)) => return,
        _ => panic!("Failed"),
    };
}

#[test]
fn div_integer() {

    let var_name = "z";

    let inst = ControlASM::Div(
        Variable::Constant(VariableValue::Integer(6)),
        Variable::Constant(VariableValue::Integer(2)),
        Variable::Instance(var_name.to_owned()));

    let clock_server = Arc::new(ClockServer::new(120.0, 4.0));
    let clock = clock_server.into();

    let instance_vars = &mut HashMap::new();

    inst.execute(
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        instance_vars,
        &clock,
    );

    match instance_vars.get(var_name) {
        Some(VariableValue::Integer(3)) => return,
        _ => panic!("Failed"),
    };
}

#[test]
fn div_integer_zero() {

    let var_name = "z";

    let inst = ControlASM::Div(
        Variable::Constant(VariableValue::Integer(6)),
        Variable::Constant(VariableValue::Integer(0)),
        Variable::Instance(var_name.to_owned()));

    let clock_server = Arc::new(ClockServer::new(120.0, 4.0));
    let clock = clock_server.into();

    let instance_vars = &mut HashMap::new();

    inst.execute(
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        instance_vars,
        &clock,
    );

    match instance_vars.get(var_name) {
        Some(VariableValue::Integer(0)) => return,
        _ => panic!("Failed"),
    };
}

#[test]
fn mod_float() {

    let var_name = "z";

    let inst = ControlASM::Mod(
        Variable::Constant(VariableValue::Float(6.0)),
        Variable::Constant(VariableValue::Float(2.0)),
        Variable::Instance(var_name.to_owned()));

    let clock_server = Arc::new(ClockServer::new(120.0, 4.0));
    let clock = clock_server.into();

    let instance_vars = &mut HashMap::new();

    inst.execute(
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        instance_vars,
        &clock,
    );

    match instance_vars.get(var_name) {
        Some(VariableValue::Float(0.0)) => return,
        _ => panic!("Failed"),
    };
}

#[test]
fn mod_integer() {

    let var_name = "z";

    let inst = ControlASM::Mod(
        Variable::Constant(VariableValue::Integer(7)),
        Variable::Constant(VariableValue::Integer(4)),
        Variable::Instance(var_name.to_owned()));

    let clock_server = Arc::new(ClockServer::new(120.0, 4.0));
    let clock = clock_server.into();

    let instance_vars = &mut HashMap::new();

    inst.execute(
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        instance_vars,
        &clock,
    );

    match instance_vars.get(var_name) {
        Some(VariableValue::Integer(3)) => return,
        _ => panic!("Failed"),
    };
}

#[test]
fn mod_integer_zero() {

    let var_name = "z";

    let inst = ControlASM::Mod(
        Variable::Constant(VariableValue::Integer(7)),
        Variable::Constant(VariableValue::Integer(0)),
        Variable::Instance(var_name.to_owned()));

    let clock_server = Arc::new(ClockServer::new(120.0, 4.0));
    let clock = clock_server.into();

    let instance_vars = &mut HashMap::new();

    inst.execute(
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        instance_vars,
        &clock,
    );

    match instance_vars.get(var_name) {
        Some(VariableValue::Integer(7)) => return,
        _ => panic!("Failed"),
    };
}

#[test]
fn mul_float() {

    let var_name = "z";

    let inst = ControlASM::Mul(
        Variable::Constant(VariableValue::Float(6.0)),
        Variable::Constant(VariableValue::Float(2.0)),
        Variable::Instance(var_name.to_owned()));

    let clock_server = Arc::new(ClockServer::new(120.0, 4.0));
    let clock = clock_server.into();

    let instance_vars = &mut HashMap::new();

    inst.execute(
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        instance_vars,
        &clock,
    );

    match instance_vars.get(var_name) {
        Some(VariableValue::Float(12.0)) => return,
        _ => panic!("Failed"),
    };
}

#[test]
fn mul_integer() {

    let var_name = "z";

    let inst = ControlASM::Mul(
        Variable::Constant(VariableValue::Integer(6)),
        Variable::Constant(VariableValue::Integer(2)),
        Variable::Instance(var_name.to_owned()));

    let clock_server = Arc::new(ClockServer::new(120.0, 4.0));
    let clock = clock_server.into();

    let instance_vars = &mut HashMap::new();

    inst.execute(
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        instance_vars,
        &clock,
    );

    match instance_vars.get(var_name) {
        Some(VariableValue::Integer(12)) => return,
        _ => panic!("Failed"),
    };
}

#[test]
fn sub_float() {

    let var_name = "z";

    let inst = ControlASM::Sub(
        Variable::Constant(VariableValue::Float(4.0)),
        Variable::Constant(VariableValue::Float(5.0)),
        Variable::Instance(var_name.to_owned()));

    let clock_server = Arc::new(ClockServer::new(120.0, 4.0));
    let clock = clock_server.into();

    let instance_vars = &mut HashMap::new();

    inst.execute(
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        instance_vars,
        &clock,
    );

    match instance_vars.get(var_name) {
        Some(VariableValue::Float(-1.0)) => return,
        _ => panic!("Failed"),
    };
}

#[test]
fn sub_integer() {

    let var_name = "z";

    let inst = ControlASM::Sub(
        Variable::Constant(VariableValue::Integer(4)),
        Variable::Constant(VariableValue::Integer(5)),
        Variable::Instance(var_name.to_owned()));

    let clock_server = Arc::new(ClockServer::new(120.0, 4.0));
    let clock = clock_server.into();

    let instance_vars = &mut HashMap::new();

    inst.execute(
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        instance_vars,
        &clock,
    );

    match instance_vars.get(var_name) {
        Some(VariableValue::Integer(-1)) => return,
        _ => panic!("Failed"),
    };
}

// No jump in boolean operations
#[test]
fn boolean_no_jump() {

    let var_name = "z";

    let inst = ControlASM::And(
        Variable::Constant(VariableValue::Bool(true)),
        Variable::Constant(VariableValue::Bool(true)),
        Variable::Instance(var_name.to_owned()));

    let clock_server = Arc::new(ClockServer::new(120.0, 4.0));
    let clock = clock_server.into();

    let instance_vars = &mut HashMap::new();

    let index = inst.execute(
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        instance_vars,
        &clock,
    );

    match index {
        None => return,
        Some(_) => panic!("Failed"),
    };
}

// Type casting in boolean operations
#[test]
fn boolean_cast() {

    let var_name = "z";

    let inst = ControlASM::And(
        Variable::Constant(VariableValue::Integer(4)),
        Variable::Constant(VariableValue::Float(5.0)),
        Variable::Instance(var_name.to_owned()));

    let clock_server = Arc::new(ClockServer::new(120.0, 4.0));
    let clock = clock_server.into();

    let instance_vars = &mut HashMap::new();

    inst.execute(
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        instance_vars,
        &clock,
    );

    match instance_vars.get(var_name) {
        Some(VariableValue::Bool(_)) => return,
        _ => panic!("Failed"),
    };
}

// Result of boolean operations
#[test]
fn and() {

    let var_tt = "a";
    let var_tf = "b";
    let var_ft = "c";
    let var_ff = "d";

    let clock_server = Arc::new(ClockServer::new(120.0, 4.0));
    let clock = clock_server.into();

    let instance_vars = &mut HashMap::new();

    let inst_tt = ControlASM::And(
        Variable::Constant(VariableValue::Bool(true)),
        Variable::Constant(VariableValue::Bool(true)),
        Variable::Instance(var_tt.to_owned()));

    let inst_tf = ControlASM::And(
        Variable::Constant(VariableValue::Bool(true)),
        Variable::Constant(VariableValue::Bool(false)),
        Variable::Instance(var_tf.to_owned()));

    let inst_ft = ControlASM::And(
        Variable::Constant(VariableValue::Bool(false)),
        Variable::Constant(VariableValue::Bool(true)),
        Variable::Instance(var_ft.to_owned()));
    
    let inst_ff = ControlASM::And(
        Variable::Constant(VariableValue::Bool(false)),
        Variable::Constant(VariableValue::Bool(false)),
        Variable::Instance(var_ff.to_owned()));

    inst_tt.execute(
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        instance_vars,
        &clock,
    );

    inst_tf.execute(
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        instance_vars,
        &clock,
    );

    inst_ft.execute(
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        instance_vars,
        &clock,
    );

    inst_ff.execute(
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        instance_vars,
        &clock,
    );

    match (instance_vars.get(var_tt), instance_vars.get(var_tf), instance_vars.get(var_ft), instance_vars.get(var_ff)) {
        (Some(VariableValue::Bool(true)),
        Some(VariableValue::Bool(false)),
        Some(VariableValue::Bool(false)),
        Some(VariableValue::Bool(false))) => return,
        _ => panic!("Failed"),
    };
}

#[test]
fn or() {

    let var_tt = "a";
    let var_tf = "b";
    let var_ft = "c";
    let var_ff = "d";

    let clock_server = Arc::new(ClockServer::new(120.0, 4.0));
    let clock = clock_server.into();

    let instance_vars = &mut HashMap::new();

    let inst_tt = ControlASM::Or(
        Variable::Constant(VariableValue::Bool(true)),
        Variable::Constant(VariableValue::Bool(true)),
        Variable::Instance(var_tt.to_owned()));

    let inst_tf = ControlASM::Or(
        Variable::Constant(VariableValue::Bool(true)),
        Variable::Constant(VariableValue::Bool(false)),
        Variable::Instance(var_tf.to_owned()));

    let inst_ft = ControlASM::Or(
        Variable::Constant(VariableValue::Bool(false)),
        Variable::Constant(VariableValue::Bool(true)),
        Variable::Instance(var_ft.to_owned()));
    
    let inst_ff = ControlASM::Or(
        Variable::Constant(VariableValue::Bool(false)),
        Variable::Constant(VariableValue::Bool(false)),
        Variable::Instance(var_ff.to_owned()));

    inst_tt.execute(
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        instance_vars,
        &clock,
    );

    inst_tf.execute(
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        instance_vars,
        &clock,
    );

    inst_ft.execute(
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        instance_vars,
        &clock,
    );

    inst_ff.execute(
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        instance_vars,
        &clock,
    );

    match (instance_vars.get(var_tt), instance_vars.get(var_tf), instance_vars.get(var_ft), instance_vars.get(var_ff)) {
        (Some(VariableValue::Bool(true)),
        Some(VariableValue::Bool(true)),
        Some(VariableValue::Bool(true)),
        Some(VariableValue::Bool(false))) => return,
        _ => panic!("Failed"),
    };
}

#[test]
fn xor() {

    let var_tt = "a";
    let var_tf = "b";
    let var_ft = "c";
    let var_ff = "d";

    let clock_server = Arc::new(ClockServer::new(120.0, 4.0));
    let clock = clock_server.into();

    let instance_vars = &mut HashMap::new();

    let inst_tt = ControlASM::Xor(
        Variable::Constant(VariableValue::Bool(true)),
        Variable::Constant(VariableValue::Bool(true)),
        Variable::Instance(var_tt.to_owned()));

    let inst_tf = ControlASM::Xor(
        Variable::Constant(VariableValue::Bool(true)),
        Variable::Constant(VariableValue::Bool(false)),
        Variable::Instance(var_tf.to_owned()));

    let inst_ft = ControlASM::Xor(
        Variable::Constant(VariableValue::Bool(false)),
        Variable::Constant(VariableValue::Bool(true)),
        Variable::Instance(var_ft.to_owned()));
    
    let inst_ff = ControlASM::Xor(
        Variable::Constant(VariableValue::Bool(false)),
        Variable::Constant(VariableValue::Bool(false)),
        Variable::Instance(var_ff.to_owned()));

    inst_tt.execute(
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        instance_vars,
        &clock,
    );

    inst_tf.execute(
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        instance_vars,
        &clock,
    );

    inst_ft.execute(
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        instance_vars,
        &clock,
    );

    inst_ff.execute(
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        instance_vars,
        &clock,
    );

    match (instance_vars.get(var_tt), instance_vars.get(var_tf), instance_vars.get(var_ft), instance_vars.get(var_ff)) {
        (Some(VariableValue::Bool(false)),
        Some(VariableValue::Bool(true)),
        Some(VariableValue::Bool(true)),
        Some(VariableValue::Bool(false))) => return,
        _ => panic!("Failed"),
    };
}


#[test]
fn not() {

    let var_t = "a";
    let var_f = "b";

    let clock_server = Arc::new(ClockServer::new(120.0, 4.0));
    let clock = clock_server.into();

    let instance_vars = &mut HashMap::new();

    let inst_t = ControlASM::Not(
        Variable::Constant(VariableValue::Bool(true)),
        Variable::Instance(var_t.to_owned()));

    let inst_f = ControlASM::Not(
        Variable::Constant(VariableValue::Bool(false)),
        Variable::Instance(var_f.to_owned()));

    inst_t.execute(
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        instance_vars,
        &clock,
    );

    inst_f.execute(
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        instance_vars,
        &clock,
    );

    match (instance_vars.get(var_t), instance_vars.get(var_f)) {
        (Some(VariableValue::Bool(false)),
        Some(VariableValue::Bool(true))) => return,
        _ => panic!("Failed"),
    };
}

// No jump in bitwise operations
#[test]
fn bitwise_no_jump() {

    let var_name = "z";

    let inst = ControlASM::BitAnd(
        Variable::Constant(VariableValue::Integer(1)),
        Variable::Constant(VariableValue::Integer(1)),
        Variable::Instance(var_name.to_owned()));

    let clock_server = Arc::new(ClockServer::new(120.0, 4.0));
    let clock = clock_server.into();

    let instance_vars = &mut HashMap::new();

    let index = inst.execute(
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        instance_vars,
        &clock,
    );

    match index {
        None => return,
        Some(_) => panic!("Failed"),
    };
}

// Type casting in bitwise operations
#[test]
fn bitwise_cast() {

    let var_name = "z";

    let inst = ControlASM::BitAnd(
        Variable::Constant(VariableValue::Bool(true)),
        Variable::Constant(VariableValue::Float(5.0)),
        Variable::Instance(var_name.to_owned()));

    let clock_server = Arc::new(ClockServer::new(120.0, 4.0));
    let clock = clock_server.into();

    let instance_vars = &mut HashMap::new();

    inst.execute(
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        instance_vars,
        &clock,
    );

    match instance_vars.get(var_name) {
        Some(VariableValue::Integer(_)) => return,
        _ => panic!("Failed"),
    };
}

// Result of bitwise operations
#[test]
fn bitand() {

    let var_name = "z";

    let inst = ControlASM::BitAnd(
        Variable::Constant(VariableValue::Integer(6)),
        Variable::Constant(VariableValue::Integer(3)),
        Variable::Instance(var_name.to_owned()));

    let clock_server = Arc::new(ClockServer::new(120.0, 4.0));
    let clock = clock_server.into();

    let instance_vars = &mut HashMap::new();

    inst.execute(
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        instance_vars,
        &clock,
    );

    match instance_vars.get(var_name) {
        Some(VariableValue::Integer(2)) => return,
        _ => panic!("Failed"),
    };
}

#[test]
fn bitor() {

    let var_name = "z";

    let inst = ControlASM::BitOr(
        Variable::Constant(VariableValue::Integer(6)),
        Variable::Constant(VariableValue::Integer(3)),
        Variable::Instance(var_name.to_owned()));

    let clock_server = Arc::new(ClockServer::new(120.0, 4.0));
    let clock = clock_server.into();

    let instance_vars = &mut HashMap::new();

    inst.execute(
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        instance_vars,
        &clock,
    );

    match instance_vars.get(var_name) {
        Some(VariableValue::Integer(7)) => return,
        _ => panic!("Failed"),
    };
}

#[test]
fn bitxor() {

    let var_name = "z";

    let inst = ControlASM::BitXor(
        Variable::Constant(VariableValue::Integer(6)),
        Variable::Constant(VariableValue::Integer(3)),
        Variable::Instance(var_name.to_owned()));

    let clock_server = Arc::new(ClockServer::new(120.0, 4.0));
    let clock = clock_server.into();

    let instance_vars = &mut HashMap::new();

    inst.execute(
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        instance_vars,
        &clock,
    );

    match instance_vars.get(var_name) {
        Some(VariableValue::Integer(5)) => return,
        _ => panic!("Failed"),
    };
}

#[test]
fn shiftleft() {

    let var_name = "z";

    let inst = ControlASM::ShiftLeft(
        Variable::Constant(VariableValue::Integer(6)),
        Variable::Constant(VariableValue::Integer(3)),
        Variable::Instance(var_name.to_owned()));

    let clock_server = Arc::new(ClockServer::new(120.0, 4.0));
    let clock = clock_server.into();

    let instance_vars = &mut HashMap::new();

    inst.execute(
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        instance_vars,
        &clock,
    );

    match instance_vars.get(var_name) {
        Some(VariableValue::Integer(48)) => return,
        _ => panic!("Failed"),
    };
}

#[test]
fn shiftrighta() {

    let var_name = "z";

    let inst = ControlASM::ShiftRightA(
        Variable::Constant(VariableValue::Integer(-8)),
        Variable::Constant(VariableValue::Integer(2)),
        Variable::Instance(var_name.to_owned()));

    let clock_server = Arc::new(ClockServer::new(120.0, 4.0));
    let clock = clock_server.into();

    let instance_vars = &mut HashMap::new();

    inst.execute(
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        instance_vars,
        &clock,
    );

    match instance_vars.get(var_name) {
        Some(VariableValue::Integer(-2)) => return,
        _ => panic!("Failed"),
    };
}

#[test]
fn shiftrightl() {

    let var_name = "z";

    let inst = ControlASM::ShiftRightL(
        Variable::Constant(VariableValue::Integer(-8)),
        Variable::Constant(VariableValue::Integer(2)),
        Variable::Instance(var_name.to_owned()));

    let clock_server = Arc::new(ClockServer::new(120.0, 4.0));
    let clock = clock_server.into();

    let instance_vars = &mut HashMap::new();

    inst.execute(
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        instance_vars,
        &clock,
    );

    match instance_vars.get(var_name) {
        Some(VariableValue::Integer(n)) => {
            if *n != i64::MAX/2 - 1 {
                panic!("Failed");
            }
        },
        _ => panic!("Failed"),
    };
}

#[test]
fn bitnot() {

    let var_name = "z";

    let inst = ControlASM::BitNot(
        Variable::Constant(VariableValue::Integer(-8)),
        Variable::Instance(var_name.to_owned()));

    let clock_server = Arc::new(ClockServer::new(120.0, 4.0));
    let clock = clock_server.into();

    let instance_vars = &mut HashMap::new();

    inst.execute(
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        instance_vars,
        &clock,
    );

    match instance_vars.get(var_name) {
        Some(VariableValue::Integer(7)) => return,
        _ => panic!("Failed"),
    };
}

// Jumps
#[test]
fn jump() {

    let goal_index = 12;

    let inst = ControlASM::Jump(goal_index);

    let clock_server = Arc::new(ClockServer::new(120.0, 4.0));
    let clock = clock_server.into();

    let index = inst.execute(
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &clock,
    );

    match index {
        Some(n) => {
            if n != goal_index {
                panic!("Failed");
            }
        },
        _ => panic!("Failed"),
    };
}

#[test]
fn jumpif() {

    let goal_index = 12;

    let inst_jump = ControlASM::JumpIf(
        Variable::Constant(VariableValue::Bool(true)),
        goal_index);

    let inst_no_jump = ControlASM::JumpIf(
        Variable::Constant(VariableValue::Bool(false)),
        goal_index);


    let clock_server = Arc::new(ClockServer::new(120.0, 4.0));
    let clock = clock_server.into();

    let index_jump = inst_jump.execute(
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &clock,
    );

    let index_no_jump = inst_no_jump.execute(
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &clock,
    );

    match (index_jump, index_no_jump) {
        (Some(n), None) => {
            if n != goal_index {
                panic!("Failed");
            }
        },
        _ => panic!("Failed"),
    };
}

#[test]
fn jumpifequal() {

    let goal_index = 12;

    let inst_jump = ControlASM::JumpIfEqual(
        Variable::Constant(VariableValue::Integer(1)),
        Variable::Constant(VariableValue::Integer(1)),
        goal_index);

    let inst_no_jump = ControlASM::JumpIfEqual(
        Variable::Constant(VariableValue::Integer(0)),
        Variable::Constant(VariableValue::Integer(1)),
        goal_index);


    let clock_server = Arc::new(ClockServer::new(120.0, 4.0));
    let clock = clock_server.into();

    let index_jump = inst_jump.execute(
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &clock,
    );

    let index_no_jump = inst_no_jump.execute(
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &clock,
    );

    match (index_jump, index_no_jump) {
        (Some(n), None) => {
            if n != goal_index {
                panic!("Failed");
            }
        },
        _ => panic!("Failed"),
    };
}

#[test]
fn jumpifless() {

    let goal_index = 12;

    let inst_jump = ControlASM::JumpIfLess(
        Variable::Constant(VariableValue::Integer(1)),
        Variable::Constant(VariableValue::Integer(2)),
        goal_index);

    let inst_border = ControlASM::JumpIfLess(
        Variable::Constant(VariableValue::Integer(1)),
        Variable::Constant(VariableValue::Integer(1)),
        goal_index);

    let inst_no_jump = ControlASM::JumpIfLess(
        Variable::Constant(VariableValue::Integer(1)),
        Variable::Constant(VariableValue::Integer(0)),
        goal_index);


    let clock_server = Arc::new(ClockServer::new(120.0, 4.0));
    let clock = clock_server.into();

    let index_jump = inst_jump.execute(
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &clock,
    );

    let index_border = inst_border.execute(
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &clock,
    );

    let index_no_jump = inst_no_jump.execute(
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &clock,
    );

    match (index_jump, index_border, index_no_jump) {
        (Some(n), None, None) => {
            if n != goal_index {
                panic!("Failed");
            }
        },
        _ => panic!("Failed"),
    };
}

#[test]
fn jumpiflessorequal() {

    let goal_index = 12;

    let inst_jump = ControlASM::JumpIfLessOrEqual(
        Variable::Constant(VariableValue::Integer(1)),
        Variable::Constant(VariableValue::Integer(2)),
        goal_index);

    let inst_border = ControlASM::JumpIfLessOrEqual(
        Variable::Constant(VariableValue::Integer(1)),
        Variable::Constant(VariableValue::Integer(1)),
        goal_index);

    let inst_no_jump = ControlASM::JumpIfLessOrEqual(
        Variable::Constant(VariableValue::Integer(1)),
        Variable::Constant(VariableValue::Integer(0)),
        goal_index);


    let clock_server = Arc::new(ClockServer::new(120.0, 4.0));
    let clock = clock_server.into();

    let index_jump = inst_jump.execute(
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &clock,
    );

    let index_border = inst_border.execute(
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &clock,
    );

    let index_no_jump = inst_no_jump.execute(
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &mut HashMap::new(),
        &clock,
    );

    match (index_jump, index_border, index_no_jump) {
        (Some(n), Some(n_border), None) => {
            if n != goal_index || n_border != goal_index {
                panic!("Failed");
            }
        },
        _ => panic!("Failed"),
    };
}