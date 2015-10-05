#![feature(rustc_private)]

mod lexer;
#[macro_use]
mod parser;
mod expand;
mod interpreter;

#[macro_use]
extern crate log;

use std::io::{Read, stdin};

const KEYWORDS: [&'static str; 5] = ["+", "fn", "let", "macro", "print"];

fn lex(input: &str) {
    let toks = lexer::lex(input);
    for t in &toks {
        print!("{} ", t);
    }
    println!("");
    println!("{:?}", toks);
}

fn parse(input: &str) {
    let toks = lexer::lex(input);
    let ast = parser::parse(&toks);

    println!("{:?}", ast);
}

fn print(input: &str) {
    let toks = lexer::lex(input);
    let ast = parser::parse(&toks);

    println!("{}", ast);
}

fn unhygienic(input: &str) {
    let toks = lexer::lex(input);
    let ast = parser::parse(&toks);
    let ast = expand::fold(ast, &mut expand::Unhygienic::new());
    println!("{}", ast);
    let result = interpreter::run_program(&ast);

    println!("{:?}", result);    
}

fn run(input: &str) {
    let toks = lexer::lex(input);
    let ast = parser::parse(&toks);
    let result = interpreter::run_program(&ast);

    println!("{:?}", result);
}

fn main() {
    let args: Vec<_> = std::env::args().collect();

    if args.len() <= 1 {
        println!("no action provided");
        println!("  usage: s [action]");
        return;
    }

    let mut input = String::new();
    let result = stdin().read_to_string(&mut input);
    assert!(result.is_ok(), "Reading stdin failed");

    match &*args[1] {
        "lex" => lex(&input),
        "parse" => parse(&input),
        "print" => print(&input),
        "run" => run(&input),
        "expand" => unhygienic(&input),
        a => println!("unknown action: {}", a),
    }
}
