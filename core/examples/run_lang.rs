//! Generic language runner for testing Sova languages.
//!
//! Usage:
//!   cargo run -p core --example run_lang -- <lang> <source>
//!   cargo run -p core --example run_lang -- <lang> --file <path>
//!   echo "source" | cargo run -p core --example run_lang -- <lang> --stdin
//!
//! Examples:
//!   cargo run -p core --example run_lang -- bob "X 42"
//!   cargo run -p core --example run_lang -- bali "(note 60)"
//!   cargo run -p core --example run_lang -- boinx "60"

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io::{self, Read};

use sova_core::compiler::Compiler;
use sova_core::lang::bali::BaliCompiler;
use sova_core::lang::bob::BobCompiler;
use sova_core::lang::boinx::{parse_boinx, BoinxInterpreter};
use sova_core::vm::interpreter::Interpreter;
use sova_core::vm::runner::{execute_interpreter, execute_program};

enum LangType {
    Compiled(Box<dyn Compiler>),
    Interpreted(&'static str),
}

fn get_lang(lang: &str) -> Option<LangType> {
    match lang {
        "bob" => Some(LangType::Compiled(Box::new(BobCompiler))),
        "bali" => Some(LangType::Compiled(Box::new(BaliCompiler))),
        "boinx" => Some(LangType::Interpreted("boinx")),
        _ => None,
    }
}

fn print_usage() {
    eprintln!("Usage: run_lang <lang> <source>");
    eprintln!("       run_lang <lang> --file <path>");
    eprintln!("       run_lang <lang> --stdin");
    eprintln!();
    eprintln!("Languages: bob, bali, boinx");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        print_usage();
        std::process::exit(1);
    }

    let lang = &args[1];
    let source = if args[2] == "--file" {
        if args.len() < 4 {
            eprintln!("Error: --file requires a path");
            std::process::exit(1);
        }
        fs::read_to_string(&args[3]).unwrap_or_else(|e| {
            eprintln!("Error reading file: {}", e);
            std::process::exit(1);
        })
    } else if args[2] == "--stdin" {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf).unwrap_or_else(|e| {
            eprintln!("Error reading stdin: {}", e);
            std::process::exit(1);
        });
        buf
    } else {
        args[2..].join(" ")
    };

    let Some(lang_type) = get_lang(lang) else {
        eprintln!("Unknown language: {}", lang);
        eprintln!("Available: bob, bali, boinx");
        std::process::exit(1);
    };

    println!("--- SOURCE ({}) ---", lang);
    println!("{}", source);
    println!();

    match lang_type {
        LangType::Compiled(compiler) => {
            match compiler.compile(&source, &BTreeMap::new()) {
                Ok(prog) => {
                    println!("--- BYTECODE ({} instructions) ---", prog.len());
                    for (i, inst) in prog.iter().enumerate() {
                        println!("{:4}: {:?}", i, inst);
                    }
                    println!();
                    run_and_print(execute_program(prog));
                }
                Err(e) => {
                    eprintln!("--- COMPILATION ERROR ---");
                    eprintln!("{}", e.info);
                    eprintln!("At: {}-{}", e.from, e.to);
                    std::process::exit(1);
                }
            }
        }
        LangType::Interpreted(name) => {
            match name {
                "boinx" => {
                    match parse_boinx(&source) {
                        Ok(prog) => {
                            println!("--- PARSED (boinx) ---");
                            println!("{:?}", prog);
                            println!();
                            let interp: Box<dyn Interpreter> = Box::new(BoinxInterpreter::from(prog));
                            run_and_print(execute_interpreter(interp));
                        }
                        Err(e) => {
                            eprintln!("--- PARSE ERROR ---");
                            eprintln!("{}", e);
                            std::process::exit(1);
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
    }
}

fn run_and_print(result: sova_core::vm::runner::ExecutionResult) {
    println!("--- EXECUTION ---");
    println!("Events: {}", result.events.len());
    for (i, (event, time)) in result.events.iter().enumerate() {
        println!("  [{}] t={}: {:?}", i, time, event);
    }

    if result.global_vars.iter().next().is_some() {
        println!("Global vars:");
        for (name, value) in result.global_vars.iter() {
            println!("  {} = {:?}", name, value);
        }
    }

    println!("Time: {} us", result.total_time);
}
