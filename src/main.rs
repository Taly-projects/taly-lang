use std::process::exit;

use colored::Colorize;

use crate::{util::{source_file::SourceFile, position::Positioned, reference::MutRef}, lexer::{tokens::Token, lexer::Lexer}, parser::{parser::Parser, node::Node}, ir::{output::IROutput, ir::IRGenerator}, symbolizer::{symbolizer::Symbolizer, scope::{Scope, ScopeType}}, checker::checker::Checker};

pub mod util;
pub mod lexer;
pub mod parser;
pub mod ir;
pub mod symbolizer;
pub mod checker;

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

fn symbolize(src: &SourceFile, ir_output: IROutput) -> Scope {
    let mut symbolizer = Symbolizer::new(ir_output);
    match symbolizer.symbolize() {
        Ok(output) => output,
        Err(err) => {
            err.print_error(src);
            exit(4);
        },
    }
}

fn check(src: &SourceFile, ir_output: IROutput, scope: MutRef<Scope>) -> IROutput {
    let mut checker = Checker::new(ir_output, scope);
    match checker.check() {
        Ok(output) => output,
        Err(err) => {
            err.print_error(src);
            exit(4);
        },
    }
}

fn main() {
    let src = read_file("res/main.taly");
    
    // Lexer
    println!("{}", "\n/> Lexer".truecolor(81, 255, 255));
    let tokens = tokenize(&src);

    for token in tokens.iter() {
        println!("{:?}", token);
    }
    println!("\n");

    // Parser
    println!("{}", "\n/> Parser".truecolor(81, 255, 255));
    let ast = parse(&src, tokens);

    for node in ast.iter() {
        println!("{:#?}", node);
    }
    println!("\n");

    // IR Generator
    println!("{}", "\n/> IR Generator".truecolor(81, 255, 255));
    let ir_output = ir_generate(&src, ast);

    for include in ir_output.includes.iter() {
        println!("{:?}", include);
    }

    for node in ir_output.ast.iter() {
        println!("{:#?}", node);
    }
    println!("\n");

    // Symbolizer
    println!("{}", "\n/> Symbolizer".truecolor(81, 255, 255));
    let mut root_scope = symbolize(&src, ir_output.clone());
    println!("{:#?}\n", root_scope);

    // Re-process root-reference
    // TODO: Ugly: Fix (Possible Fix: Create the root in the main and pass it to the symbolizer)
    let root_scope_ref = MutRef::new(&mut root_scope);
    if let ScopeType::Root { children } = &mut root_scope.scope {
        for child in children.iter_mut() {
            child.parent = Some(root_scope_ref.clone());
        }
    } else {
        unreachable!()
    }
    
    // Checker
    println!("{}", "\n/> Checker".truecolor(81, 255, 255));
    let checker_output = check(&src, ir_output, MutRef::new(&mut root_scope));

    for include in checker_output.includes.iter() {
        println!("{:?}", include);
    }

    for node in checker_output.ast.iter() {
        println!("{:#?}", node);
    }
    println!("\n");
    
}
