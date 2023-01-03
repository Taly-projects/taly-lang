use std::process::exit;

use crate::{util::{source_file::SourceFile, position::Positioned}, lexer::{tokens::Token, lexer::Lexer}};

pub mod util;
pub mod lexer;

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
            // TODO: print error
            exit(2);
        },
    }
}

fn main() {
    let src = read_file("res/main.taly");
    
    let tokens = tokenize(&src);

    for token in tokens.iter() {
        println!("{:?}", token);
    }
}
