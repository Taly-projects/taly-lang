use crate::{util::position::Positioned, ir::{error::IRError, output::{IROutput, Include, IncludeType}}, parser::node::{Node, ValueNode, Operator, VarType}};

////////////////////////////////////////////////////////////////////////////////////////////////////
//                                           IR Generator                                         //
//////////////////////////////////////////////////////////////////////////////////////////////////// 

pub struct IRGenerator {
    ast: Vec<Positioned<Node>>,
    index: usize,
    temp_id: usize,
}

impl IRGenerator {

    pub fn new(ast: Vec<Positioned<Node>>) -> Self {
        Self {
            ast,
            index: 0,
            temp_id: 0
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

    fn generate_value(&mut self, node: Positioned<Node>) -> Result<Vec<Positioned<Node>>, IRError> {
        let Node::Value(value) = node.data.clone() else {
            unreachable!()
        };

        match value {
            ValueNode::String(str) => Ok(vec![node.convert(Node::Value(ValueNode::String(str.clone())))]),
            ValueNode::Bool(b) => Ok(vec![node.convert(Node::Value(ValueNode::Bool(b)))]),
            ValueNode::Integer(num) => Ok(vec![node.convert(Node::Value(ValueNode::Integer(num)))]),
            ValueNode::Decimal(num) => Ok(vec![node.convert(Node::Value(ValueNode::Decimal(num)))]),
        }
    }

    fn generate_function_call(&mut self, node: Positioned<Node>) -> Result<Vec<Positioned<Node>>, IRError> {
        let Node::FunctionCall { name, parameters } = node.data.clone() else {
            unreachable!()
        };

        Ok(vec![node.convert(Node::FunctionCall { 
            name: name.clone(), 
            parameters: parameters.clone()
        })])
    }

    fn generate_function_definition(&mut self, node: Positioned<Node>) -> Result<Vec<Positioned<Node>>, IRError> {
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
                        let mut child_checked = self.generate_function_definition_body(child.clone())?;
                        let child_last = child_checked.pop().unwrap();
                        new_body.append(&mut child_checked);

                        new_body.push(child.convert(Node::Return(Some(Box::new(child_last)))));
                    }
                    Node::Return(_) => new_body.append(&mut self.generate_function_definition_body(child.clone())?),
                    _ => return Err(IRError::UnexpectedNode(node, Some("expression".to_string()))),
                }
            } else {
                new_body.append(&mut self.generate_function_definition_body(child.clone())?);
            }
        }

        Ok(vec![node.convert(Node::FunctionDefinition { 
            name: name.clone(), 
            external: external.clone(), 
            parameters: parameters.clone(), 
            return_type: return_type.clone(), 
            body: new_body 
        })])
    }

    fn generate_function_definition_body(&mut self, node: Positioned<Node>) -> Result<Vec<Positioned<Node>>, IRError> {
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

    fn generate_variable_definition(&mut self, node: Positioned<Node>) -> Result<Vec<Positioned<Node>>, IRError> {
        let Node::VariableDefinition { var_type, name, data_type, value } = node.data.clone() else {
            unreachable!()
        };

        let mut pre = Vec::new();
        let value_checked = if let Some(value) = value {
            let mut value_checked = self.generate_expr(*value)?;
            let value_last = value_checked.pop().unwrap();
            pre.append(&mut value_checked);
            Some(Box::new(value_last))
        } else {
            None
        };

        pre.push(node.convert(Node::VariableDefinition { 
            var_type, 
            name, 
            data_type, 
            value: value_checked 
        }));

        Ok(pre)
    }

    fn generate_expr(&mut self, node: Positioned<Node>) -> Result<Vec<Positioned<Node>>, IRError> {
        match node.data {
            Node::Value(_) => self.generate_value(node),
            Node::FunctionCall { .. } => self.generate_function_call(node),
            Node::VariableCall(_) => self.generate_variable_call(node),
            Node::BinaryOperation { .. } => self.generate_binary_operator(node),
            _ => Err(IRError::UnexpectedNode(node, Some("Expression".to_string()))),
        }
    }

    fn generate_variable_call(&mut self, node: Positioned<Node>) -> Result<Vec<Positioned<Node>>, IRError> {
        let Node::VariableCall(name) = node.data.clone() else {
            unreachable!()
        };

        Ok(vec![node.convert(Node::VariableCall(name))])
    }

    fn generate_binary_operator(&mut self, node: Positioned<Node>) -> Result<Vec<Positioned<Node>>, IRError> {
        let Node::BinaryOperation { lhs, operator, rhs } = node.data.clone() else {
            unreachable!()
        };

        let mut lhs_gen = self.generate_expr(*lhs)?;
        let mut rhs_gen = self.generate_expr(*rhs)?;

        let mut pre = Vec::new();
        
        let lhs_last = lhs_gen.pop().unwrap();
        pre.append(&mut lhs_gen);
        let rhs_last = rhs_gen.pop().unwrap();
        pre.append(&mut rhs_gen);

        if operator.data == Operator::Assign {
            let id = format!("_temp{}", self.temp_id);
            pre.push(node.convert(Node::VariableDefinition { 
                var_type: node.convert(VarType::Constant), 
                name: node.convert(id.clone()), 
                data_type: None, 
                value: Some(Box::new(lhs_last.clone())) 
            }));
            self.temp_id += 1;

            pre.push(node.convert(Node::BinaryOperation { 
                lhs: Box::new(lhs_last.clone()), 
                operator, 
                rhs: Box::new(rhs_last) 
            }));

            pre.push(node.convert(Node::VariableCall(id.clone())));
        } else {
            pre.push(node.convert(Node::BinaryOperation { 
                lhs: Box::new(lhs_last), 
                operator, 
                rhs: Box::new(rhs_last) 
            }));
        }

        // TODO: transform (5 + a = 2) to
        // const _a = a;
        // a = 2
        // 5 + _a

        Ok(pre)
    }

    fn generate_return(&mut self, node: Positioned<Node>) -> Result<Vec<Positioned<Node>>, IRError> {
        let Node::Return(expr) = node.data.clone() else {
            unreachable!()
        };

        let mut pre = Vec::new();
        let expr_gen = if let Some(expr) = expr {
            let mut expr_gen = self.generate_expr(*expr)?;
            let expr_last = expr_gen.pop().unwrap();
            pre.append(&mut expr_gen);

            Some(Box::new(expr_last))
        } else {
            None
        };

        pre.push(node.convert(Node::Return(expr_gen)));

        Ok(pre)
    }

    pub fn generate(&mut self) -> Result<IROutput, IRError> {
        let mut output = IROutput {
            includes: Vec::new(),
            ast: Vec::new(),
        };

        while let Some(current) = self.current() {
            match current.data {
                Node::FunctionDefinition { .. } => output.ast.append(&mut self.generate_function_definition(current)?),
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