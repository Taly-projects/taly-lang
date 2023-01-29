use std::process::exit;

use crate::{util::{source_file::SourceFile, position::Positioned}, lexer::{tokens::Token, lexer::Lexer}, parser::{parser::Parser, node::Node}, ir::{output::IROutput, ir::IRGenerator}};

pub mod util;
pub mod lexer;
pub mod parser;
pub mod ir;

fn read_file(path: &str) -> SourceFile {
    match std::fs::read_to_string(path) {
        Ok(src) => SourceFile::new(path.to_string(), src),
        Err(err) => {
            println!("Failed to read file '{}', {}", path, err);
            exit(1);
        },
    }
}

fn tokenize(src: &SourceFile) -> Vec<Positioned<Token>> {
    let mut lexer = Lexer::new(&src.src);
    match lexer.tokenize() {
        Ok(tokens) => tokens,
        Err(err) => {
            err.print_error(src);
            exit(2);
        },
    }
}

fn parse(src: &SourceFile, tokens: Vec<Positioned<Token>>) -> Vec<Positioned<Node>> {
    let mut parser = Parser::new(tokens);
    match parser.parse() {
        Ok(ast) => ast,
        Err(err) => {
            err.print_error(src);
            exit(3);
        },
    }
}

fn ir_generate(src: &SourceFile, ast: Vec<Positioned<Node>>) -> IROutput {
    let mut ir = IRGenerator::new(ast);
    match ir.generate() {
        Ok(output) => output,
        Err(err) => {
            err.print_error(src);
            exit(4);
        },
    }
}

fn main() {
    let src = read_file("res/main.taly");
    
    let tokens = tokenize(&src);

    for token in tokens.iter() {
        println!("{:?}", token);
    }
    println!("\n");

    let ast = parse(&src, tokens);

    for node in ast.iter() {
        println!("{:#?}", node);
    }
    println!("\n");

    let ir_output = ir_generate(&src, ast);

    for include in ir_output.includes.iter() {
        println!("{:?}", include);
    }

    for node in ir_output.ast.iter() {
        println!("{:#?}", node);
    }
    println!("\n");
}
