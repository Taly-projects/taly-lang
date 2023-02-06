use crate::{util::position::Positioned, ir::{error::IRError, output::{IROutput, Include, IncludeType}}, parser::node::{Node, ValueNode, Operator, VarType, FunctionDefinitionParameter, AccessModifier, ElifBranch,}};

////////////////////////////////////////////////////////////////////////////////////////////////////
//                                           IR Generator                                         //
//////////////////////////////////////////////////////////////////////////////////////////////////// 

pub struct IRGenerator {
    ast: Vec<Positioned<Node>>,
    index: usize,
    temp_id: usize,
    extra_includes: Vec<Include>,
}

impl IRGenerator {

    pub fn new(ast: Vec<Positioned<Node>>) -> Self {
        Self {
            ast,
            index: 0,
            temp_id: 0,
            extra_includes: Vec::new(),
        }
    }

    fn add_extra_include(&mut self, include: Include) {
        for already in self.extra_includes.iter() {
            if already.full_path() == include.full_path() {
                return;
            }
        }

        self.extra_includes.push(include);
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
            ValueNode::Type(str) => Ok(vec![node.convert(Node::Value(ValueNode::Type(str)))]),
        }
    }

    fn generate_function_call(&mut self, node: Positioned<Node>) -> Result<Vec<Positioned<Node>>, IRError> {
        let Node::FunctionCall { name, parameters } = node.data.clone() else {
            unreachable!()
        };

        let mut pre = Vec::new();

        let mut new_params = Vec::new();
        for param in parameters {
            let mut gen_param = self.generate_expr(param)?;
            let gen_param_last = gen_param.pop().unwrap();
            
            pre.append(&mut gen_param);
            new_params.push(gen_param_last);
        }

        Ok(vec![node.convert(Node::FunctionCall { 
            name: name.clone(), 
            parameters: new_params
        })])
    }

    fn generate_function_definition(&mut self, node: Positioned<Node>, parent_type: Option<Positioned<String>>, root: bool) -> Result<Vec<Positioned<Node>>, IRError> {
        let Node::FunctionDefinition { name, external, constructor, mut parameters, mut return_type, mut body, access } = node.data.clone() else {
            unreachable!()
        };

        if let Some(parent_type) = parent_type {
            if !constructor {
                let mut new_params = Vec::new();
                new_params.push(FunctionDefinitionParameter::new(name.convert("self".to_string()), parent_type));
                new_params.append(&mut parameters);
                parameters = new_params;
            } else {
                return_type = Some(parent_type.clone());

                self.add_extra_include(Include { 
                    include_type: IncludeType::StdExternal, 
                    path: node.convert("stdlib.h".to_string()) 
                });

                let mut new_body = Vec::new();
                new_body.push(node.convert(Node::_Unchecked(Box::new(node.convert(Node::VariableDefinition { 
                    var_type: node.convert(VarType::Constant), 
                    name: node.convert("self".to_string()), 
                    data_type: Some(parent_type.clone()), 
                    value: Some(Box::new(node.convert(Node::FunctionCall { 
                        name: node.convert("malloc".to_string()), 
                        parameters: vec![
                            node.convert(Node::FunctionCall { 
                                name: node.convert("sizeof".to_string()), 
                                parameters: vec![
                                    node.convert(Node::Value(ValueNode::Type(format!("_NOPTR_{}", parent_type.data.clone()))))
                                ] 
                            })
                        ] 
                    }))),
                    access: None 
                })))));
                new_body.append(&mut body);
                new_body.push(node.convert(Node::Return(Some(Box::new(node.convert(Node::VariableCall("self".to_string())))))));
                body = new_body;
            }
        } else if root && name.data == "main" {
            if let Some(data_type) = &mut return_type {
                if data_type.data == "I32" {
                    *data_type = data_type.convert("c_int".to_string());
                } else if data_type.data != "c_int" {
                    return Err(IRError::MainFunctionShouldReturnCInt(data_type.convert(())));
                }
            } else {
                return_type = Some(name.convert("c_int".to_string()));
                body.push(name.convert(Node::Return(Some(Box::new(name.convert(Node::Value(ValueNode::Integer("0".to_string()))))))));
            }
        }

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
            constructor: constructor.clone(),
            parameters: parameters.clone(), 
            return_type: return_type.clone(), 
            body: new_body,
            access
        })])
    }

    fn generate_function_definition_body(&mut self, node: Positioned<Node>) -> Result<Vec<Positioned<Node>>, IRError> {
        match node.data.clone() {
            Node::Value(_) => self.generate_value(node),
            Node::FunctionCall { .. } => self.generate_function_call(node),
            Node::VariableDefinition { access, .. } => {
                if access.is_some() {
                    Err(IRError::CannotSpecifyAccessHere(node.convert(())))
                } else {
                    self.generate_variable_definition(node)
                }
            }
            Node::VariableCall(_) => self.generate_variable_call(node),
            Node::BinaryOperation { .. } => self.generate_binary_operator(node, false),
            Node::UnaryOperation { .. } => self.generate_unary_operator(node),
            Node::Return(_) => self.generate_return(node),
            Node::IfStatement { .. } => self.generate_if_statement(node),
            Node::WhileLoop { .. } => self.generate_while_loop(node),
            Node::MatchStatement { .. } => self.generate_match_statement(node),
            Node::_Unchecked(_) => Ok(vec![node]),
            _ => Err(IRError::UnexpectedNode(node, None)),
        }
    }

    fn generate_variable_definition(&mut self, node: Positioned<Node>) -> Result<Vec<Positioned<Node>>, IRError> {
        let Node::VariableDefinition { var_type, name, data_type, value, access } = node.data.clone() else {
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
            value: value_checked,
            access
        }));

        Ok(pre)
    }

    fn generate_expr(&mut self, node: Positioned<Node>) -> Result<Vec<Positioned<Node>>, IRError> {
        match node.data {
            Node::Value(_) => self.generate_value(node),
            Node::FunctionCall { .. } => self.generate_function_call(node),
            Node::VariableCall(_) => self.generate_variable_call(node),
            Node::BinaryOperation { .. } => self.generate_binary_operator(node, true),
            Node::UnaryOperation { .. } => self.generate_unary_operator(node),
            _ => Err(IRError::UnexpectedNode(node, Some("Expression".to_string()))),
        }
    }

    fn generate_variable_call(&mut self, node: Positioned<Node>) -> Result<Vec<Positioned<Node>>, IRError> {
        let Node::VariableCall(name) = node.data.clone() else {
            unreachable!()
        };

        Ok(vec![node.convert(Node::VariableCall(name))])
    }

    fn generate_binary_operator(&mut self, node: Positioned<Node>, used: bool) -> Result<Vec<Positioned<Node>>, IRError> {
        let Node::BinaryOperation { lhs, operator, rhs } = node.data.clone() else {
            unreachable!()
        };

        let mut lhs_gen = self.generate_expr(*lhs)?;
        let mut rhs_gen = self.generate_expr(*rhs)?;

        let mut pre = Vec::new();
        
        let lhs_last = lhs_gen.pop().unwrap();
        pre.append(&mut lhs_gen);
        let mut rhs_last = rhs_gen.pop().unwrap();
        pre.append(&mut rhs_gen);

        if operator.data == Operator::Assign && used {
            let id = format!("_temp{}", self.temp_id);
            pre.push(node.convert(Node::VariableDefinition { 
                var_type: node.convert(VarType::Constant), 
                name: node.convert(id.clone()), 
                data_type: None, 
                value: Some(Box::new(lhs_last.clone())),
                access: None
            }));
            self.temp_id += 1;

            pre.push(node.convert(Node::BinaryOperation { 
                lhs: Box::new(lhs_last.clone()), 
                operator, 
                rhs: Box::new(rhs_last) 
            }));

            pre.push(node.convert(Node::VariableCall(id.clone())));
        } else if operator.data == Operator::Access {
            if let Node::FunctionCall { parameters, .. } = &mut rhs_last.data {
                let mut new_params = Vec::new();
                new_params.push(node.convert(Node::_Optional(Box::new(lhs_last.clone()))));
                new_params.append(parameters);
                *parameters = new_params;

                pre.push(node.convert(Node::BinaryOperation { 
                    lhs: Box::new(lhs_last), 
                    operator, 
                    rhs: Box::new(rhs_last) 
                }));
            } else {
                pre.push(node.convert(Node::BinaryOperation { 
                    lhs: Box::new(lhs_last), 
                    operator, 
                    rhs: Box::new(rhs_last) 
                }));
            }
        } else if operator.data == Operator::BooleanXor {
            // lhs xor rhs => lhs || rhs && !(lhs && rhs)
            pre.push(node.convert(Node::BinaryOperation { 
                lhs: Box::new(node.convert(Node::BinaryOperation { 
                    lhs: Box::new(lhs_last.clone()), 
                    operator: operator.convert(Operator::BooleanOr), 
                    rhs: Box::new(rhs_last.clone()) 
                })), 
                operator: operator.convert(Operator::BooleanAnd), 
                rhs: Box::new(node.convert(Node::UnaryOperation { 
                    operator: operator.convert(Operator::BooleanNot), 
                    value: Box::new(node.convert(Node::BinaryOperation { 
                        lhs: Box::new(lhs_last.clone()), 
                        operator: operator.convert(Operator::BooleanAnd), 
                        rhs: Box::new(rhs_last.clone()) 
                    })) 
                })) 
            }))
        } else {
            pre.push(node.convert(Node::BinaryOperation { 
                lhs: Box::new(lhs_last), 
                operator, 
                rhs: Box::new(rhs_last) 
            }));
        }

        Ok(pre)
    }

    fn generate_unary_operator(&mut self, node: Positioned<Node>) -> Result<Vec<Positioned<Node>>, IRError> {
        let Node::UnaryOperation { operator, value } = node.data.clone() else {
            unreachable!()
        };

        let mut value_gen = self.generate_expr(*value)?;
        let value_gen_last = value_gen.pop().unwrap();
        value_gen.push(node.convert(Node::UnaryOperation { 
            operator, 
            value: Box::new(value_gen_last) 
        }));

        Ok(value_gen)
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

    fn generate_class_definition(&mut self, node: Positioned<Node>) -> Result<Vec<Positioned<Node>>, IRError> {
        let Node::ClassDefinition { name, body, access } = node.data.clone() else {
            unreachable!()
        };

        let mut new_body = Vec::new();
        let mut destructor = None;
        let mut has_constructor = false;
        let mut has_fields = false;
        for node in body.iter() {
            if let Node::FunctionDefinition { name: function_name, return_type, parameters, constructor, .. } = &node.data {
                if *constructor {
                    has_constructor = true;
                }
                if function_name.data == "destroy" {
                    if let Some(destructor) = destructor {
                        return Err(IRError::DestructorAlreadyDefined(node.convert(()), destructor));
                    } 
                    destructor = Some(node.convert(()));

                    // Check destructor
                    if return_type.is_some() {
                        return Err(IRError::DestructorShouldNotReturnAnything(node.convert(())));
                    }
    
                    if !parameters.is_empty() {
                        return Err(IRError::DestructorShouldNotHaveParameters(node.convert(())));
                    }

                    if *constructor {
                        return Err(IRError::DestructorShouldNotBeConstructor(node.convert(())));
                    }
                }

            } else if let Node::VariableDefinition { .. } = &node.data {
                has_fields = true;
            }
            new_body.append(&mut self.generate_class_definition_body(node.clone(), name.clone())?);
        }

        // Generate destructor
        if destructor.is_none() {
            new_body.append(&mut self.generate_class_definition_body(name.convert(Node::FunctionDefinition { 
                name: name.convert("destroy".to_string()), 
                external: false, 
                constructor: false, 
                parameters: vec![], 
                return_type: None, 
                body: vec![
                    name.convert(Node::_Unchecked(Box::new(name.convert(Node::FunctionCall { 
                        name: name.convert("free".to_string()), 
                        parameters: vec![
                            name.convert(Node::VariableCall("self".to_string()))
                        ] 
                    }))))
                ],
                access: Some(node.convert(AccessModifier::Public)) 
            }), name.clone())?);
        }

        // Generate default constructor
        if !has_constructor && !has_fields {
            new_body.append(&mut self.generate_class_definition_body(name.convert(Node::FunctionDefinition { 
                name: name.convert("create".to_string()), 
                external: false, 
                constructor: true, 
                parameters: vec![], 
                return_type: None, 
                body: vec![], 
                access: Some(node.convert(AccessModifier::Public)) 
            }), name.clone())?);
        }

        Ok(vec![node.convert(Node::ClassDefinition { 
            name, 
            body: new_body,
            access
        })])
    }

    fn generate_class_definition_body(&mut self, node: Positioned<Node>, parent_type: Positioned<String>) -> Result<Vec<Positioned<Node>>, IRError> {
        match node.data {
            Node::FunctionDefinition { .. } => self.generate_function_definition(node, Some(parent_type), false),
            Node::VariableDefinition { .. } => self.generate_variable_definition(node),
            Node::_Unchecked(_) => Ok(vec![node]),
            _ => Err(IRError::UnexpectedNode(node, None)),
        }
    }

    fn generate_space_definition(&mut self, node: Positioned<Node>) -> Result<Vec<Positioned<Node>>, IRError> {
        let Node::SpaceDefinition { name, body, access } = node.data.clone() else {
            unreachable!()
        };

        let mut new_body = Vec::new();
        for node in body.iter() {
            new_body.append(&mut self.generate_space_definition_body(node.clone())?);
        }

        Ok(vec![node.convert(Node::SpaceDefinition { 
            name, 
            body: new_body,
            access
        })])
    }

    fn generate_space_definition_body(&mut self, node: Positioned<Node>) -> Result<Vec<Positioned<Node>>, IRError> {
        match node.data {
            Node::FunctionDefinition { constructor, .. } if !constructor => self.generate_function_definition(node, None, false),
            Node::ClassDefinition { .. } => self.generate_class_definition(node),
            Node::SpaceDefinition { .. } => self.generate_space_definition(node),
            Node::_Unchecked(_) => Ok(vec![node]),
            _ => Err(IRError::UnexpectedNode(node, None)),
        }
    }

    fn generate_if_statement(&mut self, node: Positioned<Node>) -> Result<Vec<Positioned<Node>>, IRError> {
        let Node::IfStatement { condition, body, elif_branches, else_body } = node.data.clone() else {
            unreachable!()
        };

        let mut pre = Vec::new();

        let mut gen_condition = self.generate_expr(*condition)?;
        let gen_condition_last = gen_condition.pop().unwrap();
        pre.append(&mut gen_condition);

        let mut gen_body = Vec::new();
        for node in body {
            gen_body.append(&mut self.generate_function_definition_body(node)?);
        }

        let mut elif_branch_gen = Vec::new();
        for elif_branch in elif_branches {
            let mut gen_condition = self.generate_expr(elif_branch.condition)?;
            let gen_condition_last = gen_condition.pop().unwrap();
            pre.append(&mut gen_condition);

            let mut gen_body = Vec::new();
            for node in elif_branch.body {
                gen_body.append(&mut self.generate_function_definition_body(node)?);
            }

            elif_branch_gen.push(ElifBranch {
                condition: gen_condition_last,
                body: gen_body
            });
        }

        let mut gen_else_body = Vec::new();
        for node in else_body {
            gen_else_body.append(&mut self.generate_function_definition_body(node)?);
        }

        pre.push(node.convert(Node::IfStatement { 
            condition: Box::new(gen_condition_last), 
            body: gen_body, 
            elif_branches: elif_branch_gen, 
            else_body: gen_else_body 
        }));

        Ok(pre)
    }

    fn generate_while_loop(&mut self, node: Positioned<Node>) -> Result<Vec<Positioned<Node>>, IRError> {
        let Node::WhileLoop { condition, body } = node.data.clone() else {
            unreachable!()
        };

        let mut pre = Vec::new();

        let mut gen_condition = self.generate_expr(*condition)?;
        let gen_condition_last = gen_condition.pop().unwrap();
        pre.append(&mut gen_condition);

        let mut gen_body = Vec::new();
        for node in body {
            gen_body.append(&mut self.generate_function_definition_body(node)?);
        }

        pre.push(node.convert(Node::WhileLoop { 
            condition: Box::new(gen_condition_last), 
            body: gen_body 
        }));

        Ok(pre)
    }

    fn generate_match_statement(&mut self, node: Positioned<Node>) -> Result<Vec<Positioned<Node>>, IRError> {
        let Node::MatchStatement { expr, branches, else_body } = node.data.clone() else {
            unreachable!()
        };

        let mut pre = Vec::new();
        let mut gen_expr = self.generate_expr(*expr)?;
        let gen_expr_last = gen_expr.pop().unwrap();
        pre.append(&mut gen_expr);

        if branches.is_empty() {
            return Err(IRError::CannotHaveEmptyMatchExpression(node.convert(())));
        }

        let mut if_condition = None;
        let mut if_body = Vec::new();

        let mut gen_branches = Vec::new();
        for branch in branches {
            let mut gen_condition = self.generate_expr(branch.condition)?;
            let gen_condition_last = gen_condition.pop().unwrap();
            pre.append(&mut gen_condition);

            let mut gen_body = Vec::new();
            for node in branch.body {
                gen_body.append(&mut self.generate_function_definition_body(node)?);
            }

            if if_condition.is_none() {
                if_condition = Some(gen_condition_last.clone().convert(Node::BinaryOperation { 
                    lhs: Box::new(gen_expr_last.clone()),
                    operator: gen_condition_last.convert(Operator::Equal), 
                    rhs: Box::new(gen_condition_last) 
                }));
                if_body = gen_body;
            } else {
                gen_branches.push(ElifBranch {
                    condition: gen_condition_last.clone().convert(Node::BinaryOperation { 
                        lhs: Box::new(gen_expr_last.clone()),
                        operator: gen_condition_last.convert(Operator::Equal), 
                        rhs: Box::new(gen_condition_last) 
                    }),
                    body: gen_body
                });
            }
        }

        let mut else_body_gen = Vec::new();
        for node in else_body {
            else_body_gen.append(&mut self.generate_function_definition_body(node)?);
        }

        pre.push(node.convert(Node::IfStatement { 
            condition: Box::new(if_condition.unwrap()), 
            body: if_body, 
            elif_branches: gen_branches, 
            else_body: else_body_gen 
        }));

        Ok(pre)
    }

    pub fn generate(&mut self) -> Result<IROutput, IRError> {
        let mut output = IROutput {
            includes: Vec::new(),
            ast: Vec::new(),
        };

        while let Some(current) = self.current() {
            match current.data {
                Node::FunctionDefinition { constructor, .. } if !constructor => output.ast.append(&mut self.generate_function_definition(current, None, true)?),
                Node::ClassDefinition { .. } => output.ast.append(&mut self.generate_class_definition(current)?),
                Node::SpaceDefinition { .. } => output.ast.append(&mut self.generate_space_definition(current)?),
                Node::_Unchecked(_) => output.ast.push(current),
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

        // Add Extra includes
        'A: for include in self.extra_includes.iter() {
            for already in output.includes.iter() {
                if already.full_path() == include.full_path() {
                    continue 'A;
                }
            }

            output.includes.push(include.clone());
        }
    
        Ok(output)
    }

}