use crate::{util::position::{Position, Positioned}, lexer::{tokens::{Token, Keyword}, error::LexerError}};

////////////////////////////////////////////////////////////////////////////////////////////////////
//                                              Lexer                                             //
//////////////////////////////////////////////////////////////////////////////////////////////////// 

pub struct Lexer {
    chars: Vec<char>,
    pos: Position
}

impl Lexer {

    pub fn new(src: &str) -> Self {
        let chars = src.chars().collect();
        Self {
            chars,
            pos: Position::default()
        }
    }

    fn peek(&self, x: usize) -> char {
        self.chars.get(self.pos.index + x).cloned().unwrap_or('\0')
    }

    fn current(&self) -> char {
        self.peek(0)
    } 

    fn advance(&mut self) {
        self.pos.advance(self.current())
    }

    fn advance_x(&mut self, x: usize) {
        for i in 0..x {
            self.advance();
        }
    }

    pub fn make_identifier(&mut self) -> Result<Positioned<Token>, LexerError> {
        let start = self.pos.clone();
        let mut buf = String::new();

        let mut current = self.current();
        while current.is_alphanumeric() || current == '_' {
            buf.push(current);
            self.advance();
            current = self.current();
        }

        let end = self.pos.clone();
        if let Some(keyword) = Keyword::from_string(&buf) {
            Ok(Positioned::new(Token::Keyword(keyword), start, end))
        } else {
            todo!("Identifier")
        }
    }

    pub fn make_string(&mut self) -> Result<Positioned<Token>, LexerError> {
        let start = self.pos.clone();
        self.advance();

        let mut buf = String::new();
        loop {
            let current = self.current();
            match current {
                '\0' => return Err(LexerError::UnexpectedEOF(Some("\"".to_string()))),
                '"' => break,
                _ => buf.push(current)                
            }
            self.advance();
        }

        let mut end = self.pos.clone();
        end.advance('"');

        Ok(Positioned::new(Token::String(buf), start, end))
    }

    pub fn tokenize(&mut self) -> Result<Vec<Positioned<Token>>, LexerError> {
        let mut tokens = Vec::new();

        loop {
            let current = self.current();
            
            match current {
                'a'..='z' | 'A'..='Z' => {
                    tokens.push(self.make_identifier()?);
                    continue;
                }
                '"' => tokens.push(self.make_string()?),
                '\0' => break,
                ' ' | '\r' => {
                    // Ignore
                }
                _ => return Err(LexerError::UnexpectedEOF(None))
            }

            self.advance();
        }

        Ok(tokens)
    }

}