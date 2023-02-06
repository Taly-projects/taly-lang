use crate::{symbolizer::{scope::{Scope, ScopeType}, trace::Trace}, ir::output::IROutput, util::{position::Positioned, reference::MutRef}, parser::node::{Node, ValueNode, Operator, VarType, AccessModifier, ElifBranch}, checker::error::CheckerError};

////////////////////////////////////////////////////////////////////////////////////////////////////
//                                            Node Info                                           //
//////////////////////////////////////////////////////////////////////////////////////////////////// 

struct NodeInfo {
    pub checked: Positioned<Node>,
    pub data_type: Option<Positioned<String>>,
    pub selected: Option<MutRef<Scope>>,
    pub function_called: Option<MutRef<Scope>>
}



////////////////////////////////////////////////////////////////////////////////////////////////////
//                                             Checker                                            //
//////////////////////////////////////////////////////////////////////////////////////////////////// 

pub struct Checker {
    ir_output: IROutput,
    scope: MutRef<Scope>,
    index: usize,
    trace: Trace,
    inferred: Vec<(Trace, Positioned<String>)>,
    selected: bool,
    base_scope: Option<MutRef<Scope>>,
    block_parent: bool,
}

impl Checker {

    pub fn new(ir_output: IROutput, scope: MutRef<Scope>) -> Self {
        Self {
            ir_output,
            scope,
            index: 0,
            trace: Trace::default(),
            inferred: Vec::new(),
            selected: false,
            base_scope: None,
            block_parent: false
        }
    }

    fn current(&self) -> Option<Positioned<Node>> {
        self.ir_output.ast.get(self.index).cloned()
    }

    fn advance(&mut self) {
        self.index += 1;
    }
    
    fn check_type(&mut self, found_node: Positioned<()>, expected: Positioned<String>, found: Option<Positioned<String>>) -> Result<(), CheckerError> {
        if let Some(found) = found.clone() {
            if found.data == expected.data {
                return Ok(());
            }

            match (found.data.as_str(), expected.data.as_str()) {
                ("c_string", "String") | ("String", "c_string") => Ok(()),
                ("c_int", "I32") | ("I32", "c_int") => Ok(()),
                ("c_float", "F32") | ("F32", "c_float") => Ok(()),
                (_, _) => Err(CheckerError::UnexpectedType(found_node.convert(Some(found.data.clone())), Some(expected)))
            }
        } else {
            Err(CheckerError::UnexpectedType(found_node.convert(None), Some(expected)))
        }
    }

    fn check_value_node(&mut self, node: Positioned<Node>) -> Result<NodeInfo, CheckerError> {
        let Node::Value(value) = node.data.clone() else {
            unreachable!()
        };

        match value {
            ValueNode::String(str) => Ok(NodeInfo {
                checked: node.convert(Node::Value(ValueNode::String(str.clone()))),
                data_type: Some(node.convert("String".to_string())),
                selected: None,
                function_called: None
            }),
            ValueNode::Bool(b) => Ok(NodeInfo {
                checked: node.convert(Node::Value(ValueNode::Bool(b))),
                data_type: Some(node.convert("Bool".to_string())),
                selected: None,
                function_called: None
            }),
            ValueNode::Integer(num) => Ok(NodeInfo {
                checked: node.convert(Node::Value(ValueNode::Integer(num))),
                data_type: Some(node.convert("I32".to_string())),
                selected: None,
                function_called: None
            }),
            ValueNode::Decimal(num) => Ok(NodeInfo {
                checked: node.convert(Node::Value(ValueNode::Decimal(num))),
                data_type: Some(node.convert("F32".to_string())),
                selected: None,
                function_called: None
            }),
            ValueNode::Type(str) => {
                let selected = match str.as_str() {
                    "c_string" | "c_int" | "c_float" => None,
                    _ => {
                        if let Some(class) = self.scope.get().get_class(self.trace.clone(), str.clone()) {
                            Some(class)
                        } else if let Some(space) = self.scope.get().get_space(self.trace.clone(), str.clone()) {
                            Some(space)
                        } else {
                            return Err(CheckerError::SymbolNotFound(node.convert(str.clone())));
                        }
                    }
                };
                Ok(NodeInfo {
                    checked: node.convert(Node::Value(ValueNode::Type(str))),
                    data_type: None,
                    selected,
                    function_called: None
                })
        },
        }
    }

    fn check_function_definition(&mut self, node: Positioned<Node>) -> Result<NodeInfo, CheckerError> {
        let Node::FunctionDefinition { name, external, constructor, parameters, return_type, body, access } = node.data.clone() else {
            unreachable!()
        };

        // Enter Scope
        let processed_name;
        if let Some(function) = self.scope.get().enter_function(self.trace.clone(), name.data.clone(), true, true) {
            function.get().parent = Some(self.scope.clone()); // FIXME: Somehow fix the problem
            self.scope = function;
            processed_name = self.scope.get().process_name();
        } else {
            unreachable!("Symbol '{}' not found", name.data);
        }

        // Check Body
        let mut new_body = Vec::new();
        self.trace = Trace::new(0, self.trace.clone());
        for child in body {
            let checked_child = self.check_node(child)?;
            new_body.push(checked_child.checked);
            self.trace.index += 1;
        }
        let parent_trace = *self.trace.clone().parent.unwrap();
        self.trace = parent_trace;

        // Exit Scope
        if let Some(parent) = self.scope.get().parent.clone() {
            self.scope = parent;
        } else {
            unreachable!("Not parent after entering function!");
        }

        Ok(NodeInfo {
            checked: node.convert(Node::_Renamed { 
                name: processed_name, 
                node: Box::new(node.convert(Node::FunctionDefinition { 
                    name, 
                    external, 
                    constructor,
                    parameters, 
                    return_type, 
                    body: new_body,
                    access
                }))}),
                data_type: None,
                selected: None,
                function_called: None
        })
    }

    fn check_function_call(&mut self, node: Positioned<Node>) -> Result<NodeInfo, CheckerError> {
        let Node::FunctionCall { name, parameters } = node.data.clone() else {
            unreachable!()
        };

        // Find scope-symbol
        let processed_name;
        let function = if self.selected {
            if let Some(function) = self.scope.get().enter_function(self.trace.clone(), name.data.clone(), true, self.scope.get().is_variable()) {
                processed_name = function.get().process_name();
                function
            } else {
                println!("a: {:#?}", self.scope.get());
                return Err(CheckerError::SymbolNotFound(name));
            }
        } else {
            if let Some(function) = self.scope.get().get_function(self.trace.clone(), name.data.clone(), false) {
                processed_name = function.get().process_name();
                function
            } else {
                println!("b: {:#?}", self.scope.get().scope);
                return Err(CheckerError::SymbolNotFound(name));
            }
        };

        let ScopeType::Function { params: def_params, return_type: def_return_type, .. } = &function.get().scope else {
            unreachable!()
        };

        // Set scope to base scope
        if let Some(base_scope) = self.base_scope.take() {
            self.scope = base_scope;
        }
        self.block_parent = false;

        // Check parameters (number + type)
        let parameters_len = parameters.len();
        let mut index = 0;
        let mut checked_parameters = Vec::new();
        for mut param in parameters {
            if let Node::_Optional(inner) = param.data {
                if parameters_len > def_params.len() {
                    continue;
                } else {
                    param = *inner;
                }
            } 
            let checked_param = self.check_node(param.clone())?;

            if let Some(def_param) = def_params.get(index) {
                self.check_type(param.convert(()), def_param.data_type.clone(), checked_param.data_type)?;
            } else {
                return Err(CheckerError::TooManyParameters(parameters_len, def_params.len(), name.clone(), function.get().pos.clone()));
            }

            checked_parameters.push(checked_param.checked);
            index += 1;
        }
        if index != def_params.len() {
            return Err(CheckerError::NotEnoughParameters(parameters_len, def_params.len(), name.clone(), function.get().pos.clone()));
        }

        return Ok(NodeInfo { 
            checked: node.convert(Node::_Renamed { 
                name: processed_name, 
                node: Box::new(node.convert(Node::FunctionCall { 
                    name, 
                    parameters: checked_parameters
                }))
            }), 
            data_type: def_return_type.clone(),
            selected: None,
            function_called: Some(function)
        })
    }

    fn check_variable_definition(&mut self, node: Positioned<Node>) -> Result<NodeInfo, CheckerError> {
        let Node::VariableDefinition { var_type, name, value, access, .. } = node.data.clone() else {
            unreachable!()
        };

        // Find scope-symbol
        let Some(variable) = self.scope.get().get_variable(self.trace.clone(), name.data.clone(), true) else {
            unreachable!("{}", name.data)
        };

        let ScopeType::Variable { data_type: def_data_type, .. } = &mut variable.get().scope else {
            unreachable!()
        };

        let value_checked = if let Some(value) = value {
            let info = self.check_node(*value.clone())?;
            if let Some(def_data_type) = def_data_type {
                // Check type
                self.check_type(value.convert(()), def_data_type.clone(), info.data_type)?;
            } else if let Some(info_data_type) = info.data_type {
                // Infer Type
                *def_data_type = Some(info_data_type.clone());
            }
            Some(Box::new(info.checked))
        } else {
            None
        };

        Ok(NodeInfo {
            checked: node.convert(Node::VariableDefinition { 
                var_type, 
                name, 
                data_type: def_data_type.clone(), 
                value: value_checked,
                access
            }), data_type: None,
            selected: None,
            function_called: None
        })
    }

    fn check_variable_call(&mut self, node: Positioned<Node>) -> Result<NodeInfo, CheckerError> {
        let Node::VariableCall(name) = node.data.clone() else {
            unreachable!()
        };

        let variable = if self.block_parent {
            self.scope.get().enter_variable(self.trace.clone(), name.clone(), true, self.scope.get().is_variable())
        } else {
            self.scope.get().get_variable(self.trace.clone(), name.clone(), self.scope.get().is_variable())
        };

        if let Some(variable) = variable {
            let ScopeType::Variable { var_type: def_var_type, name: def_name, data_type: def_data_type, initialized: def_initialized } = &variable.get().scope else {
                unreachable!()
            };
    
            if def_var_type.data == VarType::Constant && !def_initialized {
                return Err(CheckerError::VariableNotInitialized(def_name.clone()));
            }
    
            Ok(NodeInfo {
                checked: node.convert(Node::VariableCall(name.clone())),
                data_type: def_data_type.clone(),
                selected: Some(variable),
                function_called: None
            })
        } else if let Some(class) = self.scope.get().get_class(self.trace.clone(), name.clone()) {
            Ok(NodeInfo {
                checked: node.convert(Node::VariableCall(name.clone())),
                data_type: None,
                selected: Some(class),
                function_called: None
            })
        } else if let Some(space) = self.scope.get().get_space(self.trace.clone(), name.clone()) {
            Ok(NodeInfo {
                checked: node.convert(Node::VariableCall(name.clone())),
                data_type: None,
                selected: Some(space),
                function_called: None
            })
        } else {
            return Err(CheckerError::SymbolNotFound(node.convert(name)));
        }
    }

    fn check_access_modifier(&mut self, node: Positioned<()>, selected: &MutRef<Scope>) -> Result<(), CheckerError> {
        let selected = selected.get();
        
        if self.trace.follows_path(&selected.trace) {
            Ok(())
        } else if let Some(access) = &selected.access {
            match &access.data {
                AccessModifier::Public => Ok(()),
                AccessModifier::Private => Err(CheckerError::CannotAccessPrivateMember(node, selected.pos.clone())),
                AccessModifier::Protected => Err(CheckerError::CannotAccessProtectedMember(node, selected.pos.clone())),
                AccessModifier::Locked => unimplemented!(),
                AccessModifier::Guarded => unimplemented!(),
            }
        } else {
            Err(CheckerError::CannotAccessPrivateMember(node, selected.pos.clone()))
        }
    }

    fn check_assignment(&mut self, node: Positioned<Node>) -> Result<NodeInfo, CheckerError> {
        let Node::BinaryOperation { lhs, operator, rhs } = node.data.clone() else {
            unreachable!()
        };

        let checked_lhs = self.check_node(*lhs.clone())?;
        let checked_rhs = self.check_node(*rhs.clone())?;
        
        if let Some(selected) = checked_lhs.selected {
            if let ScopeType::Variable { var_type, name, data_type, initialized } = &mut selected.get().scope {
                if var_type.data == VarType::Constant && *initialized {
                    return Err(CheckerError::CannotAssignToConstant(node.convert(()), selected.get().pos.convert(name.data.clone())));
                }

                if let Some(data_type) = data_type {
                    self.check_type(rhs.convert(()), data_type.clone(), checked_rhs.data_type)?;

                    *initialized = true;
                    return Ok(NodeInfo {
                        checked: node.convert(Node::BinaryOperation { 
                            lhs: Box::new(checked_lhs.checked), 
                            operator, 
                            rhs: Box::new(checked_rhs.checked)
                        }),
                        data_type: Some(data_type.clone()),
                        selected: None,
                        function_called: None
                    });
                } else if let Some(rhs_type) = checked_rhs.data_type {
                    self.inferred.push((selected.get().trace.clone(), rhs_type.clone()));
                    *data_type = Some(rhs_type.clone());
                    *initialized = true;
                    return Ok(NodeInfo {
                        checked: node.convert(Node::BinaryOperation { 
                            lhs: Box::new(checked_lhs.checked), 
                            operator, 
                            rhs: Box::new(checked_rhs.checked)
                        }),
                        data_type: Some(rhs_type.clone()),
                        selected: None,
                        function_called: None
                    });
                } else {
                    return Err(CheckerError::CannotInferType(selected.get().pos.convert(name.data.clone())));
                }
            } else {
                return Err(CheckerError::CannotAssignToConstantExpression(node.convert(())));
            }
        } else {
            return Err(CheckerError::CannotAssignToConstantExpression(node.convert(())));
        }
    }

    fn check_access(&mut self, node: Positioned<Node>) -> Result<NodeInfo, CheckerError> {
        let Node::BinaryOperation { lhs, operator, rhs } = node.data.clone() else {
            unreachable!()
        };

        let checked_lhs = self.check_node(*lhs.clone())?;
        if let Some(selected) = checked_lhs.selected {
            let prev_scope = self.scope.clone();
            let prev_trace = self.trace.clone();
            let prev_selected = self.selected;
            let prev_block_parent = self.block_parent;
            self.scope = selected;
            self.trace = Trace::full();
            self.selected = true;
            self.block_parent = true;
            
            let base_scope_changed = if self.base_scope.is_none() {
                self.base_scope = Some(prev_scope.clone());
                true
            } else {
                false
            };

            let checked_rhs = self.check_node(*rhs.clone())?;
            
            self.scope = prev_scope;
            self.trace = prev_trace;
            self.selected = prev_selected;
            self.block_parent = prev_block_parent;

            if base_scope_changed {
                self.base_scope = None;
            }

            if let Some(selected_rhs) = &checked_rhs.selected {
                self.check_access_modifier(node.convert(()), selected_rhs)?;
            } else if let Some(function_called) = &checked_rhs.function_called {
                self.check_access_modifier(node.convert(()), function_called)?;
            }

            return Ok(NodeInfo {
                checked: node.convert(Node::BinaryOperation { 
                    lhs: Box::new(checked_lhs.checked), 
                    operator, 
                    rhs: Box::new(checked_rhs.checked) 
                }),
                data_type: checked_rhs.data_type,
                selected: checked_rhs.selected,
                function_called: checked_rhs.function_called
            })
        } else if let Some(data_type) = checked_lhs.data_type {
            if let Some(class) = self.scope.get().get_class(self.trace.clone(), data_type.data.clone()) {
                let prev_scope = self.scope.clone();
                let prev_trace = self.trace.clone();
                let prev_selected = self.selected;
                self.scope = class;
                self.trace = Trace::full();
                self.selected = true;
                
                let base_scope_changed = if self.base_scope.is_none() {
                    self.base_scope = Some(prev_scope.clone());
                    true
                } else {
                    false
                };

                let checked_rhs = self.check_node(*rhs.clone())?;
                
                self.scope = prev_scope;
                self.trace = prev_trace;
                self.selected = prev_selected;

                if base_scope_changed {
                    self.base_scope = None;
                }

                if let Some(selected_rhs) = &checked_rhs.selected {
                    self.check_access_modifier(node.convert(()), selected_rhs)?;
                } else if let Some(function_called) = &checked_rhs.function_called {
                    self.check_access_modifier(node.convert(()), function_called)?;
                }

                return Ok(NodeInfo {
                    checked: node.convert(Node::BinaryOperation { 
                        lhs: Box::new(checked_lhs.checked), 
                        operator, 
                        rhs: Box::new(checked_rhs.checked) 
                    }),
                    data_type: checked_rhs.data_type,
                    selected: checked_rhs.selected,
                    function_called: checked_rhs.function_called
                })
            } else {
                return Err(CheckerError::SymbolNotFound(data_type));
            }
        } else {
            return Err(CheckerError::CannotAccessAnythingHere(lhs.convert(())));
        }
    }

    fn check_binary_operation(&mut self, node: Positioned<Node>) -> Result<NodeInfo, CheckerError> {
        let Node::BinaryOperation { lhs, operator, rhs } = node.data.clone() else {
            unreachable!()
        };

        match operator.data {
            Operator::Add |
            Operator::Subtract |
            Operator::Multiply |
            Operator::Divide |
            Operator::BooleanAnd |
            Operator::BooleanOr |
            Operator::BooleanXor => {
                let checked_lhs = self.check_node(*lhs.clone())?;
                let checked_rhs = self.check_node(*rhs.clone())?;

                match (checked_lhs.data_type, checked_rhs.data_type) {
                    (Some(lhs_type), Some(rhs_type)) => {
                        self.check_type(rhs.convert(()), lhs_type.clone(), Some(rhs_type))?;
                        return Ok(NodeInfo {
                            checked: node.convert(Node::BinaryOperation { 
                                lhs: Box::new(checked_lhs.checked), 
                                operator, 
                                rhs: Box::new(checked_rhs.checked)
                            }),
                            data_type: Some(lhs_type),
                            selected: None,
                            function_called: None
                        })
                    }
                    (Some(_), _) => Err(CheckerError::UnexpectedType(rhs.convert(None), None)),
                    (_, _) => Err(CheckerError::UnexpectedType(lhs.convert(None), None)),
                }
                
            },
            Operator::Equal |
            Operator::NotEqual |
            Operator::Less |
            Operator::LessOrEqual |
            Operator::Greater |
            Operator::GreaterOrEqual => {
                let checked_lhs = self.check_node(*lhs.clone())?;
                let checked_rhs = self.check_node(*rhs.clone())?;

                match (checked_lhs.data_type, checked_rhs.data_type) {
                    (Some(lhs_type), Some(rhs_type)) => {
                        self.check_type(rhs.convert(()), lhs_type.clone(), Some(rhs_type))?;
                        return Ok(NodeInfo {
                            checked: node.convert(Node::BinaryOperation { 
                                lhs: Box::new(checked_lhs.checked), 
                                operator, 
                                rhs: Box::new(checked_rhs.checked)
                            }),
                            data_type: Some(node.convert("c_int".to_string())),
                            selected: None,
                            function_called: None
                        })
                    }
                    (Some(_), _) => Err(CheckerError::UnexpectedType(rhs.convert(None), None)),
                    (_, _) => Err(CheckerError::UnexpectedType(lhs.convert(None), None)),
                }
            }
            Operator::Assign => self.check_assignment(node),
            Operator::Access => self.check_access(node),
            _ => unreachable!()
        }
    }

    fn check_unary_operation(&mut self, node: Positioned<Node>) -> Result<NodeInfo, CheckerError> {
        let Node::UnaryOperation { operator, value } = node.data.clone() else {
            unreachable!()
        };

        let checked_value = self.check_node(*value)?;

        Ok(NodeInfo { 
            checked: node.convert(Node::UnaryOperation { 
                operator, 
                value: Box::new(checked_value.checked) 
            }), data_type: checked_value.data_type, 
            selected: None, 
            function_called: None 
        })
    }

    fn check_return(&mut self, node: Positioned<Node>) -> Result<NodeInfo, CheckerError> {
        let Node::Return(expr) = node.data.clone() else {
            unreachable!()
        };

        // Check Type
        let ScopeType::Function { return_type, .. } = &self.scope.get().clone().scope else {
            unreachable!()
        };

        let checked_return = if let Some(return_type) = return_type {
            if let Some(expr) = expr {
                let checked_expr = self.check_node(*expr.clone())?;
                if let Some(selected) = checked_expr.selected {
                    if let ScopeType::Variable { name, initialized, data_type, .. } = &selected.get().scope {
                        if !initialized { 
                            return Err(CheckerError::VariableNotInitialized(selected.get().pos.convert(name.data.clone())))
                        } if data_type.is_none() {
                            return Err(CheckerError::CannotInferType(selected.get().pos.convert(name.data.clone())));
                        } 
                    }
                }

                self.check_type(expr.convert(()), return_type.clone(), checked_expr.data_type)?;
                Some(Box::new(checked_expr.checked))
            } else {
                return Err(CheckerError::UnexpectedType(node.convert(None), Some(return_type.clone())));
            }
        } else if let Some(expr) = expr {
            let checked_expr = self.check_node(*expr.clone())?;
            return Err(CheckerError::UnexpectedType(node.convert(checked_expr.data_type.map(|x| x.data)), None));
        } else {
            None
        };

        Ok(NodeInfo { 
            checked: node.convert(Node::Return(checked_return)), 
            data_type: None, 
            selected: None,
            function_called: None
        })
    }

    fn check_class_definition(&mut self, node: Positioned<Node>) -> Result<NodeInfo, CheckerError> {
        let Node::ClassDefinition { name, body, access } = node.data.clone() else {
            unreachable!()
        };

        // Enter Scope
        if let Some(class) = self.scope.get().enter_class(self.trace.clone(), name.data.clone()) {
            self.scope = class;
        } else {
            unreachable!("Symbol '{}' not found", name.data);
        }

        // Check Body
        let mut new_body = Vec::new();
        self.trace = Trace::new(0, self.trace.clone());
        for child in body {
            let checked_child = self.check_node(child)?;
            new_body.push(checked_child.checked);
            self.trace.index += 1;
        }
        let parent_trace = *self.trace.clone().parent.unwrap();
        self.trace = parent_trace;

        // Exit Scope
        if let Some(parent) = self.scope.get().parent.clone() {
            self.scope = parent;
        } else {
            unreachable!("Not parent after entering function!");
        }

        Ok(NodeInfo {
            checked: node.convert(Node::ClassDefinition { 
                name, 
                body: new_body,
                access
            }),
            data_type: None,
            selected: None,
            function_called: None
        })
    }

    fn check_space_definition(&mut self, node: Positioned<Node>) -> Result<NodeInfo, CheckerError> {
        let Node::SpaceDefinition { name, body, access } = node.data.clone() else {
            unreachable!()
        };

        // Enter Scope
        if let Some(class) = self.scope.get().enter_space(self.trace.clone(), name.data.clone()) {
            self.scope = class;
        } else {
            unreachable!("Symbol '{}' not found", name.data);
        }

        // Check Body
        let mut new_body = Vec::new();
        self.trace = Trace::new(0, self.trace.clone());
        for child in body {
            let checked_child = self.check_node(child)?;
            new_body.push(checked_child.checked);
            self.trace.index += 1;
        }
        let parent_trace = *self.trace.clone().parent.unwrap();
        self.trace = parent_trace;

        // Exit Scope
        if let Some(parent) = self.scope.get().parent.clone() {
            self.scope = parent;
        } else {
            unreachable!("Not parent after entering function!");
        }

        Ok(NodeInfo {
            checked: node.convert(Node::SpaceDefinition { 
                name, 
                body: new_body,
                access
            }),
            data_type: None,
            selected: None,
            function_called: None
        })
    }

    fn check_if_statement(&mut self, node: Positioned<Node>) -> Result<NodeInfo, CheckerError> {
        let Node::IfStatement { condition, body, elif_branches, else_body } = node.data.clone() else {
            unreachable!()
        };

        let checked_condition = self.check_node(*condition)?;
        self.check_type(checked_condition.checked.convert(()), node.convert("c_int".to_string()), checked_condition.data_type)?;

        let mut checked_body = Vec::new();
        
        // Enter Scope
        self.scope = self.scope.get().get_child(self.trace.index);

        self.trace = Trace::new(0, self.trace.clone());
        for node in body {
            let checked_node = self.check_node(node)?;
            checked_body.push(checked_node.checked);
            self.trace.index += 1;
        }
        let parent_trace = *self.trace.clone().parent.unwrap();
        self.trace = parent_trace;
        self.trace.index += 1;

        // Exit Scope
        if let Some(parent) = self.scope.get().parent.clone() {
            self.scope = parent;
        } else {
            unreachable!("Not parent after entering function!");
        }

        let mut checked_elif_branches = Vec::new();
        for elif_branch in elif_branches {
            let checked_condition = self.check_node(elif_branch.condition)?;
            self.check_type(checked_condition.checked.convert(()), node.convert("c_int".to_string()), checked_condition.data_type)?;
            
            // Enter Scope
            self.scope = self.scope.get().get_child(self.trace.index);

            let mut checked_body = Vec::new();
            self.trace = Trace::new(0, self.trace.clone());
            for node in elif_branch.body {
                let checked_node = self.check_node(node)?;
                checked_body.push(checked_node.checked);
                self.trace.index += 1;
            }
            let parent_trace = *self.trace.clone().parent.unwrap();
            self.trace = parent_trace;
            self.trace.index += 1;

            // Exit Scope
            if let Some(parent) = self.scope.get().parent.clone() {
                self.scope = parent;
            } else {
                unreachable!("Not parent after entering function!");
            }

            checked_elif_branches.push(ElifBranch {
                condition: checked_condition.checked,
                body: checked_body
            })
        }

        // Enter Scope
        self.scope = self.scope.get().get_child(self.trace.index);

        let mut checked_else_body = Vec::new();
        self.trace = Trace::new(0, self.trace.clone());
        for node in else_body {
            let checked_node = self.check_node(node)?;
            checked_else_body.push(checked_node.checked);
            self.trace.index += 1;
        }
        let parent_trace = *self.trace.clone().parent.unwrap();
        self.trace = parent_trace;
        self.trace.index += 1;

        // Exit Scope
        if let Some(parent) = self.scope.get().parent.clone() {
            self.scope = parent;
        } else {
            unreachable!("Not parent after entering function!");
        }

        Ok(NodeInfo { 
            checked: node.convert(Node::IfStatement { 
                condition: Box::new(checked_condition.checked), 
                body: checked_body, 
                elif_branches: checked_elif_branches, 
                else_body: checked_else_body 
            }), 
            data_type: None, 
            selected: None, 
            function_called: None 
        })
    }

    fn check_while_loop(&mut self, node: Positioned<Node>) -> Result<NodeInfo, CheckerError> {
        let Node::WhileLoop { condition, body } = node.data.clone() else {
            unreachable!()
        };

        let checked_condition = self.check_node(*condition)?;
        self.check_type(checked_condition.checked.convert(()), node.convert("c_int".to_string()), checked_condition.data_type)?;

        let mut checked_body = Vec::new();
        
        // Enter Scope
        self.scope = self.scope.get().get_child(self.trace.index);

        self.trace = Trace::new(0, self.trace.clone());
        for node in body {
            let checked_node = self.check_node(node)?;
            checked_body.push(checked_node.checked);
            self.trace.index += 1;
        }
        let parent_trace = *self.trace.clone().parent.unwrap();
        self.trace = parent_trace;
        self.trace.index += 1;

        // Exit Scope
        if let Some(parent) = self.scope.get().parent.clone() {
            self.scope = parent;
        } else {
            unreachable!("Not parent after entering function!");
        }

        Ok(NodeInfo {
            checked: node.convert(Node::WhileLoop { 
                condition: Box::new(checked_condition.checked), 
                body: checked_body 
            }),
            data_type: None,
            selected: None,
            function_called: None,
        })
    }
 
    fn check_node(&mut self, node: Positioned<Node>) -> Result<NodeInfo, CheckerError> {
        match node.data {
            Node::Value(_) => self.check_value_node(node),
            Node::FunctionDefinition { .. } => self.check_function_definition(node),
            Node::FunctionCall { .. } => self.check_function_call(node),
            Node::Use(_) => unreachable!("Should have been separated in the IR Generator and should have panicked in the symbolizer!"),
            Node::VariableDefinition { .. } => self.check_variable_definition(node),
            Node::VariableCall(_) => self.check_variable_call(node),
            Node::BinaryOperation { .. } => self.check_binary_operation(node),
            Node::UnaryOperation { .. } => self.check_unary_operation(node),
            Node::Return(_) => self.check_return(node),
            Node::ClassDefinition { .. } => self.check_class_definition(node),
            Node::SpaceDefinition { .. } => self.check_space_definition(node),
            Node::IfStatement { .. } => self.check_if_statement(node),
            Node::WhileLoop { .. } => self.check_while_loop(node),
            Node::MatchStatement { .. } => unreachable!("Should have been processed in the IR Generator!"),
            Node::_Unchecked(inner) => Ok(NodeInfo { 
                checked: *inner, 
                data_type: None, 
                selected: None,
                function_called: None
            }),
            Node::_Optional(_) => unreachable!("Unexpected _Optional"),
            Node::_Renamed { .. } => unreachable!("Unexpected _Renamed")
        }
    }

    fn check_inference(&mut self, scope: &Scope) -> Result<(), CheckerError> {
        match &scope.scope {
            ScopeType::Root { children } |
            ScopeType::Class { children, .. } |
            ScopeType::Function { children, .. } |
            ScopeType::Space {children, .. } |
            ScopeType::Branch { children } => {
                for scope in children.iter() {
                    self.check_inference(scope)?;
                }
            },
            ScopeType::Variable { name, data_type, initialized, .. } => {
                if !initialized { 
                    return Err(CheckerError::VariableNotInitialized(scope.pos.convert(name.data.clone())))
                } if data_type.is_none() {
                    return Err(CheckerError::CannotInferType(scope.pos.convert(name.data.clone())));
                } 
            },
        }

        Ok(())
    }

    pub fn check(&mut self) -> Result<IROutput, CheckerError> {
        let root_scope = self.scope.clone(); // Saving root scope for later

        let mut output = IROutput { includes: self.ir_output.includes.clone() , ast: Vec::new() };

        while let Some(node) = self.current() {
            output.ast.push(self.check_node(node)?.checked);
            self.advance();
            self.trace.index += 1;
        }

        // Check if all variables have been initialized and types inferred
        self.check_inference(root_scope.get())?;

        // Check if types have been inferred (and set the type to the node)
        for (mut trace, data_type) in self.inferred.clone() {
            let mut trace_back = Vec::new();
            while trace.parent.is_some() {
                trace_back.push(trace.index);
                trace = *trace.clone().parent.unwrap();
            }
            trace_back.push(trace.index);

            let mut list = MutRef::new(&mut output.ast);
            for i in 0..(trace_back.len() - 1) {
                let j = (trace_back.len() - 1) - i;
                let trace_index = trace_back.get(j).unwrap();

                let node = list.get().get_mut(*trace_index).unwrap();
                match &mut node.data {
                    Node::FunctionDefinition { body, .. } => list = MutRef::new(body),
                    _ => unreachable!()
                }
            }

            let node = list.get().get_mut(trace_back[0]).unwrap();
            let Node::VariableDefinition { data_type: def_data_type, .. } = &mut node.data else {
                unreachable!()
            };

            *def_data_type = Some(data_type);
        }

        Ok(output)
    }

}