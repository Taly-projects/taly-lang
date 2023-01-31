use crate::{lexer::tokens::{Token, Keyword}, util::position::{Positioned, Position}, parser::{error::ParserError, node::{Node, ValueNode, FunctionDefinitionParameter, VarType}}};

////////////////////////////////////////////////////////////////////////////////////////////////////
//                                             Parser                                             //
//////////////////////////////////////////////////////////////////////////////////////////////////// 

pub struct Parser {
    tokens: Vec<Positioned<Token>>,
    index: usize,
    tabs: usize
}

impl Parser {

    pub fn new(tokens: Vec<Positioned<Token>>) -> Self {
        Self {
            tokens,
            index: 0,
            tabs: 0
        }
    }

    fn advance(&mut self) {
        self.index += 1;
    }

    fn advance_x(&mut self, x: usize) {
        self.index += x;
    }

    fn current(&self) -> Option<Positioned<Token>> {
        self.tokens.get(self.index).cloned()
    } 

    fn peek(&self, x: usize) -> Option<Positioned<Token>> {
        self.tokens.get(self.index + x).cloned()
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

    fn expect_id(&self) -> Result<Positioned<String>, ParserError> {
        let current = self.expect_current(Some("Identifier".to_string()))?;
        if let Token::Identifier(str) = current.data.clone() {
            Ok(current.convert(str))
        } else {
            Err(ParserError::UnexpectedToken(current, Some("Identifier".to_string())))
        }
    }

    fn expect_token(&self, token: Token) -> Result<Positioned<Token>, ParserError> {
        let current = self.expect_current(Some("String".to_string()))?;
        if token == current.data.clone() {
            Ok(current)
        } else {
            Err(ParserError::UnexpectedToken(current, Some(format!("{:?}", token))))
        }
    }

    fn parse_body(&mut self, body: &mut Vec<Positioned<Node>>) -> Result<(), ParserError> {
        // Node?([Tab][Node][\n])*
        let mut first = true;
        'A: while self.current().is_some() {
            // Tab
            let mut tab = 0;
            let pre_index = self.index;
            loop {
                if let Some(current) = self.current() {
                    if current.data == Token::Tab {
                        tab += 1;
                    } else if current.data == Token::NewLine {
                        self.advance();
                        tab = 0;
                        first = false;
                        continue;
                    } else {
                        if tab < self.tabs && !first {
                            self.index = pre_index;
                            break 'A;
                        } else {
                            break;
                        }
                    }
                    self.advance();
                } else {
                    self.index = pre_index;
                    break 'A;
                }
            }

            // Node
            println!("tabs: {:?}, current: {:?}", self.tabs, self.current());
            if let Some(node) = self.parse_current()? {
                body.push(node);
                first = false;
            }
            println!("After: tabs: {:?}, current: {:?}", self.tabs, self.current());
        }

        Ok(())
    }

    fn parse_function_call(&mut self, name: Positioned<String>) -> Result<Positioned<Node>, ParserError> {
        self.advance_x(2);

        let mut current = self.expect_current(Some(")".to_string()))?;
        let mut parameters = Vec::new();
        while current.data != Token::RightParenthesis {
            let value = self.parse_expr()?;
            parameters.push(value);

            current = self.expect_current(Some(")".to_string()))?;
            if current.data != Token::Comma {
                break;
            }
            self.advance();

            current = self.expect_current(Some(")".to_string()))?;
        }
        if current.data != Token::RightParenthesis {
            return Err(ParserError::UnexpectedToken(current, Some(")".to_string())));
        }

        let start = name.start.clone();
        let end = current.end.clone();

        Ok(Positioned::new(Node::FunctionCall { name, parameters }, start, end))
    }

    fn handle_id(&mut self, id: Positioned<String>) -> Result<Positioned<Node>, ParserError> {
        if let Some(next) = self.peek(1) {
            if Token::LeftParenthesis == next.data {
                self.parse_function_call(id)
            } else {
                todo!("Variable call");
            }
        } else {
            todo!("Variable Call")
        }
    }

    fn parse_expr0(&mut self) -> Result<Positioned<Node>, ParserError> {
        let current = self.expect_current(Some("expression".to_string()))?;
        match current.data.clone() {
            Token::String(str) => Ok(current.convert(Node::Value(ValueNode::String(str)))),
            Token::Identifier(id) => self.handle_id(current.convert(id)),
            _ => Err(ParserError::UnexpectedToken(current, Some("Expression".to_string())))
        }
    }

    fn parse_expr1(&mut self) -> Result<Positioned<Node>, ParserError> {
        let left = self.parse_expr0()?;
        self.advance();
        Ok(left)
    }

    fn parse_expr(&mut self) -> Result<Positioned<Node>, ParserError> {
        self.parse_expr1()
    }

    fn parse_use(&mut self, start: Position) -> Result<Positioned<Node>, ParserError> {
        self.advance();
        let path = self.expect_string()?;
        let end = path.end.clone();
        self.advance();
        Ok(Positioned::new(Node::Use(path), start, end))
    }

    fn parse_function_definition(&mut self, start: Position, external: bool) -> Result<Positioned<Node>, ParserError> {
        self.advance();
        let name = self.expect_id()?;
        self.advance();

        let mut parameters = Vec::new();
        self.expect_token(Token::LeftParenthesis)?;
        self.advance();
        let mut current = self.expect_current(Some(")".to_string()))?;
        while current.data != Token::RightParenthesis {
            // ID
            let param_name = self.expect_id()?;
            self.advance();
            // :
            self.expect_token(Token::Colon)?;
            self.advance();
            // Type
            let data_type = self.expect_id()?;
            self.advance();
            // Push
            parameters.push(FunctionDefinitionParameter::new(param_name, data_type));
            // ,
            current = self.expect_current(Some(")".to_string()))?;
            if current.data != Token::Comma {
                break;
            } 
            self.advance();
            current = self.expect_current(Some(")".to_string()))?;
        }
        let mut end = self.expect_token(Token::RightParenthesis)?.end;
        self.advance();

        let mut return_type = None;
        if let Some(current) = self.current() {
            if current.data == Token::Colon {
                self.advance();
                return_type = Some(self.expect_id()?);
                end = return_type.as_ref().unwrap().end.clone();
                self.advance();
            }
        }

        let mut body = Vec::new();
        if let Some(current) = self.current() {
            if current.data == Token::RightDoubleArrow {
                if external {
                    return Err(ParserError::UnexpectedToken(current, None));
                }
                self.advance();
                // Body
                self.tabs += 1;
                self.parse_body(&mut body)?;
                self.tabs -= 1;
            }
        }

        Ok(Positioned::new(Node::FunctionDefinition { 
            name, 
            external, 
            parameters, 
            return_type, 
            body 
        }, start, end))
    } 

    fn parse_variable_definition(&mut self, var_type: Positioned<VarType>) -> Result<Positioned<Node>, ParserError> {
        self.advance();
        let start = var_type.start.clone();
        
        // Name
        let name = self.expect_id()?;
        self.advance();
        let mut end = name.end.clone();

        // Type
        let mut data_type = None;
        if let Some(current) = self.current() {
            if current.data == Token::Colon {
                self.advance();
                data_type = Some(self.expect_id()?);
                end = self.current().unwrap().end.clone();
                self.advance();
            }
        }

        // Value
        let mut value = None;
        if let Some(current) = self.current() {
            if current.data == Token::Equal {
                self.advance();
                let expr = self.parse_expr()?;
                end = expr.end.clone();
                value = Some(Box::new(expr));
            }
        }


        return Ok(Positioned::new(Node::VariableDefinition { 
            var_type, 
            name, 
            data_type, 
            value 
        }, start, end))
    }

    fn handle_keyword(&mut self, keyword: Positioned<Keyword>) -> Result<Positioned<Node>, ParserError> {
        match keyword.data {
            Keyword::Use => self.parse_use(keyword.start),
            Keyword::Fn => self.parse_function_definition(keyword.start, false),
            Keyword::Extern => {
                self.advance();
                _ = self.expect_token(Token::Keyword(Keyword::Fn))?;
                self.parse_function_definition(keyword.start, true)
            },
            Keyword::Var => self.parse_variable_definition(keyword.convert(VarType::Variable)),
            Keyword::Const => self.parse_variable_definition(keyword.convert(VarType::Constant)),
        }
    }

    fn parse_current(&mut self) -> Result<Option<Positioned<Node>>, ParserError> {
        let current = self.expect_current(None)?;
        match current.data.clone() {
            Token::Keyword(keyword) => self.handle_keyword(current.convert(keyword)).map(|x| Some(x)),
            Token::String(_) | Token::Identifier(_) => self.parse_expr().map(|x| Some(x)),
            Token::NewLine | Token::Tab => {
                self.advance(); 
                Ok(None)
            }
            _ => Err(ParserError::UnexpectedToken(current, None))
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Positioned<Node>>, ParserError> {
        let mut ast = Vec::new();

        while self.current().is_some() {
            if let Some(node) = self.parse_current()? {
                ast.push(node);
            }
        }   

        Ok(ast)
    }
 
}