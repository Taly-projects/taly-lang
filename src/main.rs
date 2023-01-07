use std::process::exit;

use crate::{util::{source_file::SourceFile, position::Positioned}, lexer::{tokens::Token, lexer::Lexer}, parser::{parser::Parser, node::Node}};

pub mod util;
pub mod lexer;
pub mod parser;

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

fn main() {
    let src = read_file("res/main.taly");
    
    let tokens = tokenize(&src);

    for token in tokens.iter() {
        println!("{:?}", token);
    }
    println!("\n");

    let ast = parse(&src, tokens);

    for node in ast.iter() {
        println!("{:?}", node);
    }
    println!("\n");
}
