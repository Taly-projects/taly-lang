use std::process::exit;

use colored::Colorize;

use crate::{util::{source_file::SourceFile, position::Positioned, reference::MutRef}, lexer::{tokens::Token, lexer::Lexer}, parser::{parser::Parser, node::Node}, ir::{output::IROutput, ir::IRGenerator}, symbolizer::{symbolizer::Symbolizer, scope::{Scope}}, checker::checker::Checker, generator::{generator::Generator, project::Project}, post_processor::post_processor::PostProcessor};

pub mod util;
pub mod lexer;
pub mod parser;
pub mod ir;
pub mod symbolizer;
pub mod checker;
pub mod post_processor;
pub mod generator;

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

fn symbolize(src: &SourceFile, ir_output: IROutput, root: MutRef<Scope>) {
    let mut symbolizer = Symbolizer::new(ir_output);
    match symbolizer.symbolize(root) {
        Err(err) => {
            err.print_error(src);
            exit(4);
        },
        _ => {}
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

fn post_process(ir_output: IROutput) -> IROutput {
    let mut post_processor = PostProcessor::new(ir_output);
    post_processor.process()
}

fn generate(ir_output: IROutput) -> Project {
    let mut generator = Generator::new(ir_output);
    generator.generate()
}

fn build_project(project: Project) {
    std::fs::create_dir_all("./out/project").unwrap();
    for file in project.files.iter() {
        std::fs::write(format!("./out/project/{}.h", file.name), file.header.clone()).unwrap();
        std::fs::write(format!("./out/project/{}.c", file.name), file.src.clone()).unwrap();
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
    let mut root_scope = Scope::root();
    symbolize(&src, ir_output.clone(), MutRef::new(&mut root_scope));
    // std::fs::write("./scope_out.json", format!("{:#?}", root_scope)).unwrap();
    println!("{:#?}\n", root_scope);
    
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

    // Post processor
    println!("{}", "\n/> Post Processor".truecolor(81, 255, 255));
    let post_processor_output = post_process(checker_output);

    for include in post_processor_output.includes.iter() {
        println!("{:?}", include);
    }

    for node in post_processor_output.ast.iter() {
        println!("{:#?}", node);
    }
    println!("\n");

    // Generator
    println!("{}", "\n/> Generator".truecolor(81, 255, 255));
    let project = generate(post_processor_output);

    for file in project.files.iter() {
        println!("{}.h", file.name);
        println!("{}\n", file.header);
        println!("{}.c", file.name);
        println!("{}\n", file.src);
    }

    build_project(project);
}
