use crate::{util::position::Positioned, ir::{error::IRError, output::{IROutput, Include, IncludeType}}, parser::node::{Node, ValueNode}};

////////////////////////////////////////////////////////////////////////////////////////////////////
//                                           IR Generator                                         //
//////////////////////////////////////////////////////////////////////////////////////////////////// 

pub struct IRGenerator {
    ast: Vec<Positioned<Node>>,
    index: usize
}

impl IRGenerator {

    pub fn new(ast: Vec<Positioned<Node>>) -> Self {
        Self {
            ast,
            index: 0
        }
    }

    fn advance(&mut self) {
        self.index += 1;
    }

    fn current(&self) -> Option<Positioned<Node>> {
        self.ast.get(self.index).cloned()
    }

    fn generate_include(&mut self, path: Positioned<String>) -> Result<Include, IRError> {
        if path.data.starts_with("c-") {
            Ok(Include { 
                include_type: IncludeType::External, 
                path: path.convert(format!("{}.h", &path.data[2..path.data.len()]))
            })
        } else if path.data.starts_with("std-") {
            Ok(Include { 
                include_type: IncludeType::StdExternal, 
                path: path.convert(format!("{}.h", &path.data[4..path.data.len()]))
            })
        } else {
            Ok(Include { 
                include_type: IncludeType::Internal, 
                path: path.clone() 
            })
        }
    }

    fn generate_value(&mut self, node: Positioned<Node>) -> Result<Positioned<Node>, IRError> {
        let Node::Value(value) = node.data.clone() else {
            unreachable!()
        };

        match value {
            ValueNode::String(str) => Ok(node.convert(Node::Value(ValueNode::String(str.clone())))),
            ValueNode::Bool(b) => Ok(node.convert(Node::Value(ValueNode::Bool(b)))),
            ValueNode::Integer(num) => Ok(node.convert(Node::Value(ValueNode::Integer(num)))),
            ValueNode::Decimal(num) => Ok(node.convert(Node::Value(ValueNode::Decimal(num)))),
        }
    }

    fn generate_function_call(&mut self, node: Positioned<Node>) -> Result<Positioned<Node>, IRError> {
        let Node::FunctionCall { name, parameters } = node.data.clone() else {
            unreachable!()
        };

        Ok(node.convert(Node::FunctionCall { 
            name: name.clone(), 
            parameters: parameters.clone()
        }))
    }

    fn generate_function_definition(&mut self, node: Positioned<Node>) -> Result<Positioned<Node>, IRError> {
        let Node::FunctionDefinition { name, external, parameters, return_type, body } = node.data.clone() else {
            unreachable!()
        };

        let mut new_body = Vec::new();
        for child in body.iter() {
            if return_type.is_some() && std::ptr::eq(child, body.last().unwrap()) {
                match child.data {
                    Node::Value(_) | 
                    Node::FunctionCall { .. } | 
                    Node::VariableCall(_) |
                    Node::BinaryOperation { .. } => {
                        new_body.push(child.convert(Node::Return(Box::new(self.generate_function_definition_body(child.clone())?))));
                    }
                    Node::Return(_) => new_body.push(self.generate_function_definition_body(child.clone())?),
                    _ => return Err(IRError::UnexpectedNode(node, Some("expression".to_string()))),
                }
            } else {
                new_body.push(self.generate_function_definition_body(child.clone())?);
            }
        }

        Ok(node.convert(Node::FunctionDefinition { 
            name: name.clone(), 
            external: external.clone(), 
            parameters: parameters.clone(), 
            return_type: return_type.clone(), 
            body: new_body 
        }))
    }

    fn generate_function_definition_body(&mut self, node: Positioned<Node>) -> Result<Positioned<Node>, IRError> {
        match node.data {
            Node::Value(_) => self.generate_value(node),
            Node::FunctionDefinition { .. } => Err(IRError::UnexpectedNode(node, None)),
            Node::FunctionCall { .. } => self.generate_function_call(node),
            Node::Use(_) => Err(IRError::UnexpectedNode(node, None)),
            Node::VariableDefinition { .. } => self.generate_variable_definition(node),
            Node::VariableCall(_) => self.generate_variable_call(node),
            Node::BinaryOperation { .. } => self.generate_binary_operator(node),
            Node::Return(_) => self.generate_return(node),
        }
    }

    fn generate_variable_definition(&mut self, node: Positioned<Node>) -> Result<Positioned<Node>, IRError> {
        let Node::VariableDefinition { var_type, name, data_type, value } = node.data.clone() else {
            unreachable!()
        };

        let value_checked = if let Some(value) = value {
            Some(Box::new(self.generate_expr(*value)?))
        } else {
            None
        };

        Ok(node.convert(Node::VariableDefinition { 
            var_type, 
            name, 
            data_type, 
            value: value_checked 
        }))
    }

    fn generate_expr(&mut self, node: Positioned<Node>) -> Result<Positioned<Node>, IRError> {
        match node.data {
            Node::Value(_) => self.generate_value(node),
            Node::FunctionCall { .. } => self.generate_function_call(node),
            Node::VariableCall(_) => self.generate_variable_call(node),
            Node::BinaryOperation { .. } => self.generate_binary_operator(node),
            _ => Err(IRError::UnexpectedNode(node, Some("Expression".to_string()))),
        }
    }

    fn generate_variable_call(&mut self, node: Positioned<Node>) -> Result<Positioned<Node>, IRError> {
        let Node::VariableCall(name) = node.data.clone() else {
            unreachable!()
        };

        Ok(node.convert(Node::VariableCall(name)))
    }

    fn generate_binary_operator(&mut self, node: Positioned<Node>) -> Result<Positioned<Node>, IRError> {
        let Node::BinaryOperation { lhs, operator, rhs } = node.data.clone() else {
            unreachable!()
        };

        let lhs_gen = self.generate_expr(*lhs)?;
        let rhs_gen = self.generate_expr(*rhs)?;

        // TODO: transform (5 + a = 2) to
        // const _a = a;
        // a = 2
        // 5 + _a

        Ok(node.convert(Node::BinaryOperation { 
            lhs: Box::new(lhs_gen), 
            operator, 
            rhs: Box::new(rhs_gen) 
        }))
    }

    fn generate_return(&mut self, node: Positioned<Node>) -> Result<Positioned<Node>, IRError> {
        let Node::Return(expr) = node.data.clone() else {
            unreachable!()
        };

        let expr_gen = self.generate_expr(*expr)?;

        Ok(node.convert(Node::Return(Box::new(expr_gen))))
    }

    pub fn generate(&mut self) -> Result<IROutput, IRError> {
        let mut output = IROutput {
            includes: Vec::new(),
            ast: Vec::new(),
        };

        while let Some(current) = self.current() {
            match current.data {
                Node::FunctionDefinition { .. } => output.ast.push(self.generate_function_definition(current)?),
                Node::Use(path) => {
                    for include in output.includes.iter() {
                        if include.full_path() == format!("{}.h", path.data) {
                            return Err(IRError::FileAlreadyIncluded(path, include.path.convert(())));
                        }
                    }

                    output.includes.push(self.generate_include(path)?);
                }
                _ => return Err(IRError::UnexpectedNode(current, None)),
            }
            self.advance();
        } 
    
        Ok(output)
    }

}