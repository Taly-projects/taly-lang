use crate::{lexer::tokens::{Token, Keyword}, util::position::{Positioned, Position}, parser::{error::ParserError, node::{Node, ValueNode}}};

////////////////////////////////////////////////////////////////////////////////////////////////////
//                                             Parser                                             //
//////////////////////////////////////////////////////////////////////////////////////////////////// 

pub struct Parser {
    tokens: Vec<Positioned<Token>>,
    index: usize
}

impl Parser {

    pub fn new(tokens: Vec<Positioned<Token>>) -> Self {
        Self {
            tokens,
            index: 0
        }
    }

    fn advance(&mut self) {
        self.index += 1;
    }

    fn current(&self) -> Option<Positioned<Token>> {
        self.tokens.get(self.index).cloned()
    } 

    fn expect_current(&self, token: Option<String>) -> Result<Positioned<Token>, ParserError> {
        if let Some(current) = self.current() {
            Ok(current)
        } else {
            Err(ParserError::UnexpectedEOF(token))
        }
    } 

    fn expect_string(&self) -> Result<Positioned<String>, ParserError> {
        let current = self.expect_current(Some("String".to_string()))?;
        if let Token::String(str) = current.data.clone() {
            Ok(current.convert(str))
        } else {
            Err(ParserError::UnexpectedToken(current, Some("String".to_string())))
        }
    }

    fn parse_expr0(&mut self) -> Result<Positioned<Node>, ParserError> {
        let current = self.expect_current(Some("expression".to_string()))?;
        match current.data.clone() {
            Token::String(str) => Ok(current.convert(Node::Value(ValueNode::String(str)))),
            _ => Err(ParserError::UnexpectedToken(current, Some("Expression".to_string())))
        }
    }

    fn parse_expr(&mut self) -> Result<Positioned<Node>, ParserError> {
        self.parse_expr0()
    }

    fn parse_use(&mut self, start: Position) -> Result<Positioned<Node>, ParserError> {
        self.advance();
        let path = self.expect_string()?;
        let end = path.end.clone();
        Ok(Positioned::new(Node::Use(path), start, end))
    }

    fn handle_keyword(&mut self, keyword: Positioned<Keyword>) -> Result<Positioned<Node>, ParserError> {
        match keyword.data {
            Keyword::Use => self.parse_use(keyword.start),
        }
    }

    fn parse_current(&mut self) -> Result<Option<Positioned<Node>>, ParserError> {
        let current = self.expect_current(None)?;
        match current.data.clone() {
            Token::Keyword(keyword) => self.handle_keyword(current.convert(keyword)).map(|x| Some(x)),
            Token::String(_) => self.parse_expr().map(|x| Some(x)),
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Positioned<Node>>, ParserError> {
        let mut ast = Vec::new();

        while self.current().is_some() {
            if let Some(node) = self.parse_current()? {
                ast.push(node);
            }
            self.advance();
        }   

        Ok(ast)
    }
 
}