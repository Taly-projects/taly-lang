use crate::{lexer::tokens::{Token, Keyword}, util::position::{Positioned, Position}, parser::{error::ParserError, node::{Node, ValueNode, FunctionDefinitionParameter, VarType, Operator, AccessModifier, ElifBranch, MatchBranch, DataType}}};

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

    /* Cursor Movement */
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

    /* Expect */
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

    /* Parse */
    fn parse_body(&mut self, body: &mut Vec<Positioned<Node>>) -> Result<(), ParserError> {
        // Node?([nTab][Node][\n])* 
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
                    } else if current.data == Token::Keyword(Keyword::End) || current.data == Token::Keyword(Keyword::Else) || current.data == Token::Keyword(Keyword::Elif) {
                        break 'A;
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
            if let Some(node) = self.parse_current()? {
                body.push(node);
                first = false;
            }
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
                Ok(id.clone().convert(Node::VariableCall(id.data)))
            }
        } else {
            Ok(id.clone().convert(Node::VariableCall(id.data)))
        }
    }

    fn parse_unary(&mut self, operator: Positioned<Operator>) -> Result<Positioned<Node>, ParserError> {
        self.advance();
        let value = self.parse_expr0()?;
        let start = operator.start.clone();
        let end = value.end.clone();
        Ok(Positioned::new(Node::UnaryOperation { 
            operator, 
            value: Box::new(value) 
        }, start, end))
    }

    fn parse_expr0(&mut self) -> Result<Positioned<Node>, ParserError> {
        let current = self.expect_current(Some("expression".to_string()))?;
        match current.data.clone() {
            Token::String(str) => Ok(current.convert(Node::Value(ValueNode::String(str)))),
            Token::Bool(b) => Ok(current.convert(Node::Value(ValueNode::Bool(b)))),
            Token::Integer(num) => Ok(current.convert(Node::Value(ValueNode::Integer(num)))),
            Token::Decimal(num) => Ok(current.convert(Node::Value(ValueNode::Decimal(num)))),
            Token::Identifier(id) => self.handle_id(current.convert(id)),
            Token::Plus => self.parse_unary(current.convert(Operator::Add)),
            Token::Dash => self.parse_unary(current.convert(Operator::Subtract)),
            Token::Keyword(Keyword::Not) => self.parse_unary(current.convert(Operator::BooleanNot)),
            Token::LeftParenthesis => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect_token(Token::RightParenthesis)?;
                Ok(expr)
            }
            _ => Err(ParserError::UnexpectedToken(current, Some("Expression".to_string())))
        }
    }

    fn parse_expr1(&mut self) -> Result<Positioned<Node>, ParserError> {
        let mut left = self.parse_expr0()?;
        self.advance();

        while let Some(current) = self.current() {
            let operator = match current.data {
                Token::Dot => current.convert(Operator::Access), 
                _ => break
            };
            self.advance();

            let right = self.parse_expr0()?;
            self.advance();
            
            let start = left.start.clone();
            let end = right.end.clone();
            left = Positioned::new(Node::BinaryOperation { 
                lhs: Box::new(left), 
                operator, 
                rhs: Box::new(right)
            }, start, end);
        }

        Ok(left)
    }

    fn parse_expr2(&mut self) -> Result<Positioned<Node>, ParserError> {
        let mut left = self.parse_expr1()?;

        while let Some(current) = self.current() {
            let operator = match current.data {
                Token::Star => current.convert(Operator::Multiply), 
                Token::Slash => current.convert(Operator::Divide), 
                _ => break
            };
            self.advance();

            let right = self.parse_expr1()?;
            
            let start = left.start.clone();
            let end = right.end.clone();
            left = Positioned::new(Node::BinaryOperation { 
                lhs: Box::new(left), 
                operator, 
                rhs: Box::new(right) 
            }, start, end);
        }

        Ok(left)
    }

    fn parse_expr3(&mut self) -> Result<Positioned<Node>, ParserError> {
        let mut left = self.parse_expr2()?;

        while let Some(current) = self.current() {
            let operator = match current.data {
                Token::Plus => current.convert(Operator::Add), 
                Token::Dash => current.convert(Operator::Subtract), 
                _ => break
            };
            self.advance();

            let right = self.parse_expr2()?;
            
            let start = left.start.clone();
            let end = right.end.clone();
            left = Positioned::new(Node::BinaryOperation { 
                lhs: Box::new(left), 
                operator, 
                rhs: Box::new(right) 
            }, start, end);
        }

        Ok(left)
    }

    fn parse_expr4(&mut self) -> Result<Positioned<Node>, ParserError> {
        let mut left = self.parse_expr3()?;

        while let Some(current) = self.current() {
            let operator = match current.data {
                Token::DoubleEqual => current.convert(Operator::Equal), 
                Token::ExclamationMarkEqual => current.convert(Operator::NotEqual), 
                Token::LeftAngle => current.convert(Operator::Less), 
                Token::LeftAngleEqual => current.convert(Operator::LessOrEqual), 
                Token::RightAngle => current.convert(Operator::Greater), 
                Token::RightAngleEqual => current.convert(Operator::GreaterOrEqual), 
                _ => break
            };
            self.advance();

            let right = self.parse_expr3()?;
            
            let start = left.start.clone();
            let end = right.end.clone();
            left = Positioned::new(Node::BinaryOperation { 
                lhs: Box::new(left), 
                operator, 
                rhs: Box::new(right) 
            }, start, end);
        }

        Ok(left)
    }

    fn parse_expr5(&mut self) -> Result<Positioned<Node>, ParserError> {
        let mut left = self.parse_expr4()?;

        while let Some(current) = self.current() {
            let operator = match current.data {
                Token::Keyword(Keyword::And) => current.convert(Operator::BooleanAnd), 
                Token::Keyword(Keyword::Or) => current.convert(Operator::BooleanOr), 
                Token::Keyword(Keyword::Xor) => current.convert(Operator::BooleanXor), 
                _ => break
            };
            self.advance();

            let right = self.parse_expr4()?;
            
            let start = left.start.clone();
            let end = right.end.clone();
            left = Positioned::new(Node::BinaryOperation { 
                lhs: Box::new(left), 
                operator, 
                rhs: Box::new(right) 
            }, start, end);
        }

        Ok(left)
    }

    fn parse_expr6(&mut self) -> Result<Positioned<Node>, ParserError> {
        let mut left = self.parse_expr5()?;

        while let Some(current) = self.current() {
            let operator = match current.data {
                Token::Equal => current.convert(Operator::Assign), 
                _ => break
            };
            self.advance();

            let right = self.parse_expr()?;
            
            let start = left.start.clone();
            let end = right.end.clone();
            left = Positioned::new(Node::BinaryOperation { 
                lhs: Box::new(left), 
                operator, 
                rhs: Box::new(right) 
            }, start, end);
        }

        Ok(left)
    }

    fn parse_expr(&mut self) -> Result<Positioned<Node>, ParserError> {
        self.parse_expr6()
    }

    fn parse_use(&mut self, start: Position) -> Result<Positioned<Node>, ParserError> {
        self.advance();
        let path = self.expect_string()?;
        let end = path.end.clone();
        self.advance();
        Ok(Positioned::new(Node::Use(path), start, end))
    }

    fn parse_function_definition(&mut self, start: Position, external: bool, constructor: bool, access: Option<Positioned<AccessModifier>>) -> Result<Positioned<Node>, ParserError> {
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
            let id = self.expect_id()?;
            let data_type = id.clone().convert(DataType::Custom(id.data));
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
                let id = self.expect_id()?; // TODO: Take expr (and allow BinOp.access)
                return_type = Some(id.clone().convert(DataType::Custom(id.data)));
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

                // [Semantic]: Check the nodes inside the body
                for node in body.iter() {
                    match &node.data {
                        Node::Value(_) |
                        Node::FunctionCall { .. } |
                        Node::VariableDefinition { .. } |
                        Node::VariableCall(_) |
                        Node::BinaryOperation { .. } |
                        Node::UnaryOperation { .. } |
                        Node::Return(_) |
                        Node::IfStatement { .. } |
                        Node::WhileLoop { .. } |
                        Node::MatchStatement { .. } |
                        Node::Label { .. }  => {}
                        _ => return Err(ParserError::UnexpectedNode(node.clone(), None))
                    }
                }
            }
        }

        Ok(Positioned::new(Node::FunctionDefinition { 
            name, 
            external, 
            constructor,
            parameters, 
            return_type, 
            body,
            access 
        }, start, end))
    } 

    fn parse_variable_definition(&mut self, start: Position, var_type: Positioned<VarType>, access: Option<Positioned<AccessModifier>>) -> Result<Positioned<Node>, ParserError> {
        self.advance();

        // Name
        let name = self.expect_id()?;
        self.advance();
        let mut end = name.end.clone();

        // Type
        let mut data_type = None;
        if let Some(current) = self.current() {
            if current.data == Token::Colon {
                self.advance();
                let id= self.expect_id()?;
                data_type = Some(id.clone().convert(DataType::Custom(id.data)));
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

        if var_type.data == VarType::Constant && value.is_none() {
            return Err(ParserError::UninitializedConstant(name));
        }


        return Ok(Positioned::new(Node::VariableDefinition { 
            var_type, 
            name, 
            data_type, 
            value,
            access
        }, start, end))
    }

    fn parse_return(&mut self, pos: Positioned<()>) -> Result<Positioned<Node>, ParserError> {
        self.advance();
        if let Some(current) = self.current() {
            if current.data == Token::NewLine {
                Ok(pos.convert(Node::Return(None)))
            } else {
                let expr = self.parse_expr()?;
                let end = expr.end.clone();
                Ok(Positioned::new(Node::Return(Some(Box::new(expr))), pos.start, end))
            }
        } else {
            Ok(pos.convert(Node::Return(None)))
        }
    }

    fn parse_class_definition(&mut self, start: Position, access: Option<Positioned<AccessModifier>>) -> Result<Positioned<Node>, ParserError> {
        self.advance();
        let name = self.expect_id()?;
        self.advance();
        let mut end = name.end.clone();
        let mut extensions = Vec::new();
        if let Some(current) = self.current() {
            if current.data == Token::Colon {
                self.advance();
                loop {
                    let id = self.expect_id()?;
                    extensions.push(id);
                    self.advance();
                    if let Some(current) = self.current() {
                        if current.data != Token::Comma {
                            break;
                        } else {
                            self.advance();
                        }
                    } else {
                        break;
                    }
                }
            }
        }

        self.tabs += 1;
        let mut body = Vec::new();
        self.parse_body(&mut body)?;

        // [Semantic]: Check the nodes inside the body
        for node in body.iter() {
            match &node.data {
                Node::FunctionDefinition { .. } |
                Node::VariableDefinition { .. } => { },
                _ => return Err(ParserError::UnexpectedNode(node.clone(), Some("Method, Constructor or Field".to_string())))
            }
        }

        if let Some(last) = body.last() {
            end = last.end.clone();
        }
        self.tabs -= 1;

        Ok(Positioned::new(Node::ClassDefinition { 
            name, 
            body,
            access,
            extensions
        }, start, end))
    }

    fn parse_space_definition(&mut self, start: Position, access: Option<Positioned<AccessModifier>>) -> Result<Positioned<Node>, ParserError> {
        self.advance();
        let name = self.expect_id()?;
        self.advance();
        let mut end = name.end.clone();

        self.tabs += 1;
        let mut body = Vec::new();
        self.parse_body(&mut body)?;

        // [Semantic]: Check the nodes inside the body
        for node in body.iter() {
            match &node.data {
                Node::FunctionDefinition { .. } |
                Node::ClassDefinition { .. } |
                Node::SpaceDefinition { .. } |
                Node::InterfaceDefinition { .. } => { },
                _ => return Err(ParserError::UnexpectedNode(node.clone(), Some("Function, Class, Interface or Space".to_string())))
            }
        }

        if let Some(last) = body.last() {
            end = last.end.clone();
        }
        self.tabs -= 1;

        Ok(Positioned::new(Node::SpaceDefinition { 
            name, 
            body,
            access
        }, start, end))
    }

    fn parse_if_statement(&mut self, start: Position) -> Result<Positioned<Node>, ParserError> {
        self.advance();
        let if_condition = self.parse_expr()?;
        self.expect_token(Token::Keyword(Keyword::Then))?;
        self.advance();
        
        self.tabs += 1;
        let mut if_body = Vec::new();
        self.parse_body(&mut if_body)?;

        // [Semantic]: Check the nodes inside the body
        for node in if_body.iter() {
            match &node.data {
                Node::Value(_) |
                Node::FunctionCall { .. } |
                Node::VariableDefinition { .. } |
                Node::VariableCall(_) |
                Node::BinaryOperation { .. } |
                Node::UnaryOperation { .. } |
                Node::Return(_) |
                Node::IfStatement { .. } |
                Node::WhileLoop { .. } |
                Node::MatchStatement { .. } |
                Node::Label { .. }  => {}
                _ => return Err(ParserError::UnexpectedNode(node.clone(), None))
            }
        }

        
        let mut branches = Vec::new();
        let mut else_body = Vec::new();
        let end;

        let mut current;
        loop {
            current = self.expect_current(Some("end".to_string()))?;
            match current.data {
                Token::Keyword(Keyword::Elif) => {
                    self.advance();
                    let condition = self.parse_expr()?;
                    self.expect_token(Token::Keyword(Keyword::Then))?;
                    self.advance();

                    let mut body = Vec::new();
                    self.parse_body(&mut body)?;

                    // [Semantic]: Check the nodes inside the body
                    for node in body.iter() {
                        match &node.data {
                            Node::Value(_) |
                            Node::FunctionCall { .. } |
                            Node::VariableDefinition { .. } |
                            Node::VariableCall(_) |
                            Node::BinaryOperation { .. } |
                            Node::UnaryOperation { .. } |
                            Node::Return(_) |
                            Node::IfStatement { .. } |
                            Node::WhileLoop { .. } |
                            Node::MatchStatement { .. } |
                            Node::Label { .. }  => {}
                            _ => return Err(ParserError::UnexpectedNode(node.clone(), None))
                        }
                    }

                    branches.push(ElifBranch { 
                        condition, 
                        body 
                    })
                }
                Token::Keyword(Keyword::Else) => {
                    self.advance();

                    self.parse_body(&mut else_body)?;

                    // [Semantic]: Check the nodes inside the body
                    for node in else_body.iter() {
                        match &node.data {
                            Node::Value(_) |
                            Node::FunctionCall { .. } |
                            Node::VariableDefinition { .. } |
                            Node::VariableCall(_) |
                            Node::BinaryOperation { .. } |
                            Node::UnaryOperation { .. } |
                            Node::Return(_) |
                            Node::IfStatement { .. } |
                            Node::WhileLoop { .. } |
                            Node::MatchStatement { .. } |
                            Node::Label { .. }  => {}
                            _ => return Err(ParserError::UnexpectedNode(node.clone(), None))
                        }
                    }

                    end = self.expect_token(Token::Keyword(Keyword::End))?.end;
                    self.advance();
                    break
                }
                Token::Keyword(Keyword::End) => {
                    self.tabs -= 1;
                    end = current.end;
                    break;
                }
                _ => return Err(ParserError::UnexpectedToken(current, Some("end".to_string())))
            }
        }

        Ok(Positioned::new(Node::IfStatement { 
            condition: Box::new(if_condition), 
            body: if_body, 
            elif_branches: branches, 
            else_body 
        }, start, end))
    }

    fn parse_while_loop(&mut self, start: Position) -> Result<Positioned<Node>, ParserError> {
        self.advance();
        let expr = self.parse_expr()?;
        self.expect_token(Token::Keyword(Keyword::Do))?;
        self.advance();

        let mut body = Vec::new();
        let mut current = self.expect_current(Some("end".to_string()))?;
        while current.data != Token::Keyword(Keyword::End) {
            if let Some(node) = self.parse_current()? {
                body.push(node);
            }
            current = self.expect_current(Some("end".to_string()))?;
        }
        let end = current.end.clone();
        self.advance();

        // [Semantic]: Check the nodes inside the body
        for node in body.iter() {
            match &node.data {
                Node::Value(_) |
                Node::FunctionCall { .. } |
                Node::VariableDefinition { .. } |
                Node::VariableCall(_) |
                Node::BinaryOperation { .. } |
                Node::UnaryOperation { .. } |
                Node::Return(_) |
                Node::IfStatement { .. } |
                Node::WhileLoop { .. } |
                Node::MatchStatement { .. } |
                Node::Label { .. }  => {}
                _ => return Err(ParserError::UnexpectedNode(node.clone(), None))
            }
        }

        
        Ok(Positioned::new(Node::WhileLoop { 
            condition: Box::new(expr), 
            body 
        }, start, end))
    }

    fn parse_match_statement(&mut self, start: Position) -> Result<Positioned<Node>, ParserError> {
        self.advance();
        let expr = self.parse_expr()?;
        self.expect_token(Token::NewLine)?;
        self.advance();
        self.tabs += 1;

        let mut branches = Vec::new();
        let mut else_body = Vec::new();
        let end;

        let mut tabs = 0;
        let mut current = self.expect_current(Some("end".to_string()))?;
        loop {
            match current.data {
                Token::Keyword(Keyword::Else) => {
                    self.advance();

                    // No +1 on index because already +1 for the global match
                    self.parse_body(&mut else_body)?;

                    // [Semantic]: Check the nodes inside the body
                    for node in else_body.iter() {
                        match &node.data {
                            Node::Value(_) |
                            Node::FunctionCall { .. } |
                            Node::VariableDefinition { .. } |
                            Node::VariableCall(_) |
                            Node::BinaryOperation { .. } |
                            Node::UnaryOperation { .. } |
                            Node::Return(_) |
                            Node::IfStatement { .. } |
                            Node::WhileLoop { .. } |
                            Node::MatchStatement { .. } |
                            Node::Label { .. }  => {}
                            _ => return Err(ParserError::UnexpectedNode(node.clone(), None))
                        }
                    }
                    
                    end = self.expect_token(Token::Keyword(Keyword::End))?.end;
                    self.advance();
                    break;
                }
                Token::Keyword(Keyword::End) => {
                    end = current.end.clone();
                    self.advance();
                    break;
                }
                Token::NewLine => {
                    tabs = 0;
                    self.advance();
                }
                Token::Tab => {
                    tabs += 1;
                    self.advance();
                }
                _ => {
                    if tabs >= self.tabs {
                        tabs = 0;
                        let mut conditions = Vec::new();
                        let mut next_allowed = true;
                        // EXPR, [TAB|NL]?EXPR
                        loop {
                            let current = self.expect_current(Some("Expr or =>".to_string()))?;
                            match current.data {
                                Token::Tab | Token::NewLine => self.advance(),
                                Token::Comma if !next_allowed => {
                                    self.advance();
                                    next_allowed = true
                                } 
                                Token::RightDoubleArrow => break,
                                _ => {
                                    next_allowed = false;
                                    let condition = self.parse_expr()?;
                                    conditions.push(condition);
                                }
                            }        
                        }

                        self.expect_token(Token::RightDoubleArrow)?;
                        self.advance();
    
                        self.tabs += 1;
                        let mut body = Vec::new();
                        self.parse_body(&mut body)?;
                        self.tabs -= 1;

                        // [Semantic]: Check the nodes inside the body
                        for node in body.iter() {
                            match &node.data {
                                Node::Value(_) |
                                Node::FunctionCall { .. } |
                                Node::VariableDefinition { .. } |
                                Node::VariableCall(_) |
                                Node::BinaryOperation { .. } |
                                Node::UnaryOperation { .. } |
                                Node::Return(_) |
                                Node::IfStatement { .. } |
                                Node::WhileLoop { .. } |
                                Node::MatchStatement { .. } |
                                Node::Label { .. }  => {}
                                _ => return Err(ParserError::UnexpectedNode(node.clone(), None))
                            }
                        }
                        
                        branches.push(MatchBranch {
                            conditions,
                            body
                        });
                    } else {
                        return Err(ParserError::UnexpectedToken(current, Some("tab".to_string())));
                    }
                }  
            }
            current = self.expect_current(Some("end".to_string()))?;
        }
        self.tabs -= 1;

        Ok(Positioned::new(Node::MatchStatement { 
            expr: Box::new(expr), 
            branches, 
            else_body 
        }, start, end))
    } 

    fn parse_break(&mut self, position: Positioned<()>) -> Result<Positioned<Node>, ParserError> {
        self.advance();
        let label = if let Some(current) = self.current() {
            if let Token::Label(label) = current.data.clone() {
                self.advance();
                Some(current.convert(label))
            } else {
                None
            }
        } else {
            None
        };
        Ok(position.convert(Node::Break(label)))
    }

    fn parse_continue(&mut self, position: Positioned<()>) -> Result<Positioned<Node>, ParserError> {
        self.advance();
        let label = if let Some(current) = self.current() {
            if let Token::Label(label) = current.data.clone() {
                self.advance();
                Some(current.convert(label))
            } else {
                None
            }
        } else {
            None
        };
        Ok(position.convert(Node::Continue(label)))
    }

    fn parse_interface_definition(&mut self, start: Position, access: Option<Positioned<AccessModifier>>) -> Result<Positioned<Node>, ParserError> {
        self.advance();
        let name = self.expect_id()?;
        self.advance();
        let mut end = name.end.clone();

        self.tabs += 1;
        let mut body = Vec::new();
        self.parse_body(&mut body)?;
        if let Some(last) = body.last() {
            end = last.end.clone();
        }

        // [Semantic]: Check the nodes inside the body
        for node in body.iter() {
            match &node.data {
                Node::FunctionDefinition { .. } => {}
                _ => return Err(ParserError::UnexpectedNode(node.clone(), None))
            }
        }

        self.tabs -= 1;

        Ok(Positioned::new(Node::InterfaceDefinition { 
            name, 
            body,
            access
        }, start, end))
    }

    fn handle_access(&mut self, access: Positioned<AccessModifier>) -> Result<Positioned<Node>, ParserError> {
        self.advance();
        let current = self.expect_current(Some("Function, Class, Space, ..".to_string()))?;
        match current.data {
            Token::Keyword(Keyword::Fn) => self.parse_function_definition(access.start.clone(), false, false, Some(access)),
            Token::Keyword(Keyword::New) => self.parse_function_definition(access.start.clone(), false, true, Some(access)),
            Token::Keyword(Keyword::Var) => self.parse_variable_definition(access.start.clone(), current.convert(VarType::Variable), Some(access)),
            Token::Keyword(Keyword::Const) => self.parse_variable_definition(access.start.clone(), current.convert(VarType::Constant), Some(access)),
            Token::Keyword(Keyword::Class) => self.parse_class_definition(access.start.clone(), Some(access)),
            Token::Keyword(Keyword::Space) => self.parse_space_definition(access.start.clone(), Some(access)),
            Token::Keyword(Keyword::Intf) => self.parse_interface_definition(access.start.clone(), Some(access)),
            _ => Err(ParserError::UnexpectedToken(current, Some("Function, Class, Space, ..".to_string())))
        }
    }

    fn handle_keyword(&mut self, keyword: Positioned<Keyword>) -> Result<Positioned<Node>, ParserError> {
        match keyword.data {
            Keyword::Use => self.parse_use(keyword.start),
            Keyword::Fn => self.parse_function_definition(keyword.start, false, false, None),
            Keyword::New => self.parse_function_definition(keyword.start, false, true, None),
            Keyword::Extern => {
                self.advance();
                _ = self.expect_token(Token::Keyword(Keyword::Fn))?;
                self.parse_function_definition(keyword.start, true, false, None)
            },
            Keyword::Var => self.parse_variable_definition(keyword.start.clone(), keyword.convert(VarType::Variable), None),
            Keyword::Const => self.parse_variable_definition(keyword.start.clone(), keyword.convert(VarType::Constant), None),
            Keyword::Return => self.parse_return(keyword.convert(())),
            Keyword::Class => self.parse_class_definition(keyword.start, None),
            Keyword::Space => self.parse_space_definition(keyword.start, None),
            Keyword::Pub => self.handle_access(keyword.convert(AccessModifier::Public)),
            Keyword::Prot => self.handle_access(keyword.convert(AccessModifier::Protected)),
            Keyword::Lock => self.handle_access(keyword.convert(AccessModifier::Locked)),
            Keyword::Guard => self.handle_access(keyword.convert(AccessModifier::Guarded)),
            Keyword::If => self.parse_if_statement(keyword.start),
            Keyword::While => self.parse_while_loop(keyword.start),
            Keyword::Match => self.parse_match_statement(keyword.start),
            Keyword::Break => self.parse_break(keyword.convert(())),
            Keyword::Continue => self.parse_continue(keyword.convert(())),
            Keyword::Intf => self.parse_interface_definition(keyword.start, None),
            _ => Err(ParserError::UnexpectedToken(self.current().unwrap(), None))
        }
    }

    fn parse_label(&mut self, label: Positioned<String>) -> Result<Positioned<Node>, ParserError> {
        let start = label.start.clone();
        self.advance();
        self.expect_token(Token::Colon)?;
        self.advance();
        let current = self.expect_current(Some("loop".to_string()))?;
        match current.data {
            Token::Keyword(Keyword::While) => {
                let inner = self.parse_while_loop(current.start)?;
                let end = inner.end.clone();
                Ok(Positioned::new(Node::Label { 
                    name: label, 
                    inner: Box::new(inner) 
                }, start, end))
            },
            _ => return Err(ParserError::UnexpectedToken(current, Some("while".to_string()))),
        }
    } 

    fn parse_current(&mut self) -> Result<Option<Positioned<Node>>, ParserError> {
        let current = self.expect_current(None)?;
        match current.data.clone() {
            Token::String(_) | 
            Token::Bool(_) |
            Token::Integer(_) |
            Token::Decimal(_) |
            Token::Identifier(_) |
            Token::Plus |
            Token::Dash |
            Token::Keyword(Keyword::Not) |
            Token::LeftParenthesis => self.parse_expr().map(|x| Some(x)),
            Token::Keyword(keyword) => self.handle_keyword(current.convert(keyword)).map(|x| Some(x)),
            Token::Label(label) => self.parse_label(current.convert(label)).map(|x| Some(x)),
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