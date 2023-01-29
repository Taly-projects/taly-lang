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
        for _ in 0..x {
            self.advance();
        }
    }

    fn make_single<T>(&mut self, data: T) -> Positioned<T> {
        let start = self.pos.clone();
        let mut end = self.pos.clone();
        end.advance(self.current());
        Positioned::new(data, start, end)
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
            Ok(Positioned::new(Token::Identifier(buf), start, end))
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
        let mut space_count = 0;
        let mut space_start_pos = self.pos.clone();

        loop {
            let current = self.current();

            
            if current == ' ' {
                space_count += 1;
                if space_count == 4 {
                    let mut end = self.pos.clone();
                    end.advance(' ');
                    tokens.push(Positioned::new(Token::Tab, space_start_pos.clone(), end));
                    space_count = 0;
                } else if space_count == 1 {
                    space_start_pos = self.pos.clone();
                }
                self.advance();
                continue;
            } else {
                space_count = 0;
            }
            
            match current {
                'a'..='z' | 'A'..='Z' => {
                    tokens.push(self.make_identifier()?);
                    continue;
                }
                '"' => tokens.push(self.make_string()?),
                '(' => tokens.push(self.make_single(Token::LeftParenthesis)),
                ')' => tokens.push(self.make_single(Token::RightParenthesis)),
                ',' => tokens.push(self.make_single(Token::Comma)),
                ':' => tokens.push(self.make_single(Token::Colon)),
                '=' => {
                    let next = self.peek(1);
                    match next {
                        '>' => {
                            let start = self.pos.clone();
                            self.advance();
                            let mut end = self.pos.clone();
                            end.advance('>');
                            tokens.push(Positioned::new(Token::RightDoubleArrow, start, end));
                        }
                        _ => todo!("equal")
                    }
                }
                '\n' => {
                    let start = self.pos.clone();
                    let mut end = self.pos.clone();
                    end.advance(' ');
                    tokens.push(Positioned::new(Token::NewLine, start, end));
                },
                '\t' => tokens.push(self.make_single(Token::Tab)),
                '\0' => break,
                ' ' | '\r' => {
                    // Ignore
                }
                _ => return Err(LexerError::UnexpectedChar(self.make_single(current), None))
            }

            self.advance();
        }

        Ok(tokens)
    }

}