use crate::{ir::output::IROutput, util::position::Positioned, parser::node::{Node, Operator, ElifBranch, VarType, DataType, FunctionDefinitionParameter}};

pub struct PostProcessor {
    ir_output: IROutput,
    index: usize,
}

impl PostProcessor {

    pub fn new(ir_output: IROutput) -> Self {
        Self {
            ir_output,
            index: 0
        }
    } 

    fn current(&self) -> Option<Positioned<Node>> {
        self.ir_output.ast.get(self.index).cloned()
    }

    fn advance(&mut self) {
        self.index += 1;
    }

    fn process_function_definition(&mut self, node: Positioned<Node>, new_name: Option<String>) -> Positioned<Node> {
        let Node::FunctionDefinition { name, external, constructor, parameters, return_type, body, access  } = node.data.clone() else {
            unreachable!()
        };

        let mut new_body = Vec::new();
        for node in body {
            new_body.push(self.process_node(node, None));
        }

        node.convert(Node::FunctionDefinition { 
            name: new_name.map_or(name.clone(), |x| name.convert(x)), 
            external, 
            constructor, 
            parameters, 
            return_type, 
            body: new_body, 
            access 
        })
    }

    fn process_function_call(&mut self, node: Positioned<Node>, new_name: Option<String>) -> Positioned<Node> {
        let Node::FunctionCall { name, parameters } = node.data.clone() else {
            unreachable!()
        };

        let mut new_params = Vec::new();
        for param in parameters {
            new_params.push(self.process_node(param, None));
        }

        node.convert(Node::FunctionCall { 
            name: new_name.map_or(name.clone(), |x| name.convert(x)), 
            parameters: new_params
        })
    }

    fn process_variable_definition(&mut self, node: Positioned<Node>) -> Positioned<Node> {
        let Node::VariableDefinition { var_type, name, data_type, value, access } = node.data.clone() else {
            unreachable!()
        };

        node.convert(Node::VariableDefinition { 
            var_type, 
            name, 
            data_type, 
            value: value.map(|x| Box::new(self.process_node(*x, None))), 
            access 
        })
    }

    fn process_access(&mut self, node: Positioned<Node>) -> Positioned<Node> {
        let Node::BinaryOperation { lhs, operator, rhs } = node.data.clone() else {
            unreachable!()
        };

        if let Node::FunctionCall { .. } = rhs.data.clone() {
            self.process_node(*rhs, None)            
        } else if let Node::_Renamed { name, node } = rhs.data.clone() {
            if let Node::FunctionCall { .. } = node.data.clone() {
                self.process_node(*rhs, Some(name))            
            } else {
                node.convert(Node::BinaryOperation { 
                    lhs: Box::new(self.process_node(*lhs, None)), 
                    operator, 
                    rhs: Box::new(self.process_node(*rhs, None)) 
                })
            }
        } else {
            node.convert(Node::BinaryOperation { 
                lhs: Box::new(self.process_node(*lhs, None)), 
                operator, 
                rhs: Box::new(self.process_node(*rhs, None)) 
            })
        }
    }

    fn process_bin_op(&mut self, node: Positioned<Node>) -> Positioned<Node> {
        let Node::BinaryOperation { lhs, operator, rhs } = node.data.clone() else {
            unreachable!()
        };

        node.convert(Node::BinaryOperation { 
            lhs: Box::new(self.process_node(*lhs, None)), 
            operator, 
            rhs: Box::new(self.process_node(*rhs, None)) 
        })
    }

    fn process_unary_op(&mut self, node: Positioned<Node>) -> Positioned<Node> {
        let Node::UnaryOperation { operator, value } = node.data.clone() else {
            unreachable!()
        };

        node.convert(Node::UnaryOperation { 
            operator, 
            value: Box::new(self.process_node(*value, None)) 
        })
    }

    fn process_return(&mut self, node: Positioned<Node>) -> Positioned<Node> {
        let Node::Return(expr) = node.data.clone() else {
            unreachable!()
        };

        node.convert(Node::Return(expr.map(|x| Box::new(self.process_node(*x, None)))))
    }

    fn process_class_definition(&mut self, node: Positioned<Node>) -> Positioned<Node> {
        let Node::ClassDefinition { name, body, access, extensions } = node.data.clone() else {
            unreachable!()
        };

        let mut new_body = Vec::new();
        for node in body {
            new_body.push(self.process_node(node, None));
        }

        node.convert(Node::ClassDefinition { 
            name, 
            body: new_body, 
            access,
            extensions
        })
    }

    fn process_space_definition(&mut self, node: Positioned<Node>) -> Positioned<Node> {
        let Node::SpaceDefinition { name, body, access } = node.data.clone() else {
            unreachable!()
        };

        let mut new_body = Vec::new();
        for node in body {
            new_body.push(self.process_node(node, None));
        }

        node.convert(Node::SpaceDefinition { 
            name, 
            body: new_body, 
            access 
        })
    }

    fn process_if_statement(&mut self, node: Positioned<Node>) -> Positioned<Node> {
        let Node::IfStatement { condition, body, elif_branches, else_body } = node.data.clone() else {
            unreachable!()
        };

        let processed_condition = self.process_node(*condition, None);

        let mut processed_body = Vec::new();
        for node in body {
            processed_body.push(self.process_node(node, None));
        }

        let mut processed_elif_branches = Vec::new();
        for elif_branch in elif_branches {
            let processed_condition = self.process_node(elif_branch.condition, None);
    
            let mut processed_body = Vec::new();
            for node in elif_branch.body {
                processed_body.push(self.process_node(node, None));
            }

            processed_elif_branches.push(ElifBranch {
                condition: processed_condition,
                body: processed_body
            });
        }

        let mut processed_else_body = Vec::new();
        for node in else_body {
            processed_else_body.push(self.process_node(node, None));
        }

        node.convert(Node::IfStatement { 
            condition: Box::new(processed_condition), 
            body: processed_body, 
            elif_branches: processed_elif_branches, 
            else_body: processed_else_body 
        })
    }

    fn process_while_loop(&mut self, node: Positioned<Node>) -> Positioned<Node> {
        let Node::WhileLoop { condition, body } = node.data.clone() else {
            unreachable!()
        };

        let processed_condition = self.process_node(*condition, None);

        let mut processed_body = Vec::new();
        for node in body {
            processed_body.push(self.process_node(node, None));
        }

        node.convert(Node::WhileLoop {
            condition: Box::new(processed_condition), 
            body: processed_body, 
        })
    }

    fn process_break(&mut self, node: Positioned<Node>) -> Positioned<Node> {
        node.clone()
    }

    fn process_continue(&mut self, node: Positioned<Node>) -> Positioned<Node> {
        node.clone()
    }

    fn process_label(&mut self, node: Positioned<Node>) -> Positioned<Node> {
        let Node::Label { name, inner } = node.data.clone() else {
            unreachable!()
        };

        let processed_inner = self.process_node(*inner, None);
        
        node.convert(Node::Label {
            name,
            inner: Box::new(processed_inner)
        })
    }

    fn process_interface_definition(&mut self, node: Positioned<Node>) -> Positioned<Node> {
        let Node::InterfaceDefinition { name: interface_name, body, access } = node.data.clone() else {
            unreachable!()
        };

        let mut new_body = Vec::new();
        for node in body {
            let processed_node = self.process_node(node, None);
            let Node::FunctionDefinition { name, mut parameters, return_type, .. } = processed_node.data.clone() else {
                unreachable!("Interfaces should only contain functions!")
            };

            let mut params = Vec::new();
            params.push(interface_name.convert(DataType::Custom(format!("struct {}", interface_name.data.clone()))));
            for param in parameters.clone() {
                params.push(param.data_type);
            }

            // Field
            new_body.push(processed_node.convert(Node::VariableDefinition { 
                var_type: processed_node.convert(VarType::Constant), 
                name: name.clone(),
                data_type: Some(processed_node.convert(DataType::Function { 
                    return_type: return_type.clone().map(|x| Box::new(x)), 
                    params: params.clone() 
                })), 
                value: None, 
                access: None 
            }));

            // Process parameters
            let mut new_parameters = Vec::new();
            let mut parameters_call = Vec::new();
            new_parameters.push(FunctionDefinitionParameter {
                name: interface_name.convert("self".to_string()),
                data_type: interface_name.convert(DataType::Custom(interface_name.data.clone())),
            });
            parameters_call.push(processed_node.convert(Node::VariableCall("self".to_string())));
            for param in parameters.iter() {
                parameters_call.push(processed_node.convert(Node::VariableCall(param.name.data.clone())));
            }
            new_parameters.append(&mut parameters);

            // Methods TODO: Removed useless method (check if symbol is still present. If it is, remove it)
            // TODO: Put back needed to call a function part of the interface with only the interface and not the class
            new_body.push(processed_node.convert(Node::FunctionDefinition { 
                name: name.clone(), 
                external: false, 
                constructor: false, 
                parameters: new_parameters,
                return_type: return_type.clone(), 
                body: vec![
                    processed_node.convert(Node::BinaryOperation { 
                        lhs: Box::new(processed_node.convert(Node::VariableCall("self".to_string()))), 
                        operator: processed_node.convert(Operator::Access), 
                        rhs: Box::new(processed_node.convert(Node::FunctionCall { 
                            name: name.clone(), 
                            parameters: parameters_call
                        })) 
                    })
                ], 
                access: None 
            }))
        }

        node.convert(Node::ClassDefinition { 
            name: interface_name, 
            body: new_body, 
            access,
            extensions: Vec::new()
        })
    }

    fn process_node(&mut self, node: Positioned<Node>, new_name: Option<String>) -> Positioned<Node> {
        match node.data.clone() {
            Node::Value(_) => node,
            Node::FunctionDefinition { .. } => self.process_function_definition(node, new_name),
            Node::FunctionCall { .. } => self.process_function_call(node, new_name),
            Node::Use(_) => node,
            Node::VariableDefinition { .. } => self.process_variable_definition(node),
            Node::VariableCall(_) => node,
            Node::BinaryOperation { operator, .. } if operator.data == Operator::Access || operator.data == Operator::DotAccess => self.process_access(node),
            Node::BinaryOperation { .. } => self.process_bin_op(node),
            Node::UnaryOperation { .. } => self.process_unary_op(node),
            Node::Return(_) => self.process_return(node),
            Node::ClassDefinition { .. } => self.process_class_definition(node),
            Node::SpaceDefinition { .. } => self.process_space_definition(node),
            Node::IfStatement { .. } => self.process_if_statement(node),
            Node::WhileLoop { .. } => self.process_while_loop(node),
            Node::MatchStatement { .. } => unreachable!("Should have been processed in the IR Generator!"),
            Node::Break(_) => self.process_break(node),
            Node::Continue(_) => self.process_continue(node),
            Node::Label { .. } => self.process_label(node),
            Node::InterfaceDefinition { .. } => self.process_interface_definition(node),
            Node::_Unchecked(inner) => self.process_node(*inner, None),
            Node::_Optional(inner) => self.process_node(*inner, None),
            Node::_Renamed { name, node } => self.process_node(*node, Some(name)),
            Node::_Implementation(inner) => node.convert(Node::_Implementation(Box::new(self.process_node(*inner, new_name)))),
            Node::_Generated(_) => unreachable!("Should have been processed in the checker!")
        }
    }

    pub fn process(&mut self) -> IROutput {
        let mut output = IROutput {
            includes: self.ir_output.includes.clone(),
            ast: Vec::new(),
        };

        while let Some(current) = self.current() {
            output.ast.push(self.process_node(current, None));
            self.advance();
        }

        output
    }

}