use crate::{symbolizer::scope::{Scope, ScopeType}, ir::output::IROutput, util::{position::Positioned, reference::MutRef}, parser::node::{Node, ValueNode}, checker::error::CheckerError};

////////////////////////////////////////////////////////////////////////////////////////////////////
//                                            Node Info                                           //
//////////////////////////////////////////////////////////////////////////////////////////////////// 

struct NodeInfo {
    pub checked: Positioned<Node>,
    pub data_type: Option<Positioned<String>>,
}



////////////////////////////////////////////////////////////////////////////////////////////////////
//                                             Checker                                            //
//////////////////////////////////////////////////////////////////////////////////////////////////// 

pub struct Checker {
    ir_output: IROutput,
    scope: MutRef<Scope>,
    index: usize
}

impl Checker {

    pub fn new(ir_output: IROutput, scope: MutRef<Scope>) -> Self {
        Self {
            ir_output,
            scope,
            index: 0
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
            }),
        }
    }

    fn check_function_definition(&mut self, node: Positioned<Node>) -> Result<NodeInfo, CheckerError> {
        let Node::FunctionDefinition { name, external, parameters, return_type, body } = node.data.clone() else {
            unreachable!()
        };

        // Enter Scope
        if let Some(function) = self.scope.get().enter_function(name.data.clone()) {
            self.scope = function;
        } else {
            unreachable!("Symbol '{}' not found", name.data);
        }

        // TODO: Check return
        // Check Body
        let mut new_body = Vec::new();
        for child in body {
            let checked_child = self.check_node(child)?;
            new_body.push(checked_child.checked);
        }

        // Exit Scope
        if let Some(parent) = self.scope.get().parent.clone() {
            self.scope = parent;
        } else {
            unreachable!("Not parent after entering function!");
        }

        Ok(NodeInfo {
            checked: node.convert(Node::FunctionDefinition { 
                name, 
                external, 
                parameters, 
                return_type, 
                body: new_body 
            }),
            data_type: None,
        })
    }

    fn check_function_call(&mut self, node: Positioned<Node>) -> Result<NodeInfo, CheckerError> {
        let Node::FunctionCall { name, parameters } = node.data.clone() else {
            unreachable!()
        };

        // Find scope-symbol
        let Some(function) = self.scope.get().get_function(name.data.clone()) else {
            return Err(CheckerError::SymbolNotFound(name));
        };

        let ScopeType::Function { params: def_params, return_type: def_return_type, .. } = &function.get().scope else {
            unreachable!()
        };

        // Check parameters (number + type)
        let parameters_len = parameters.len();
        let mut index = 0;
        let mut checked_parameters = Vec::new();
        for param in parameters {
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
            checked: node.convert(Node::FunctionCall { 
                name, 
                parameters: checked_parameters
            }), 
            data_type: def_return_type.clone()
        })
    }

    fn check_variable_definition(&mut self, node: Positioned<Node>) -> Result<NodeInfo, CheckerError> {
        let Node::VariableDefinition { var_type, name, value, .. } = node.data.clone() else {
            unreachable!()
        };

        // Find scope-symbol
        let Some(variable) = self.scope.get().get_variable(name.data.clone()) else {
            return Err(CheckerError::SymbolNotFound(name));
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
                value: value_checked
            }), data_type: None,
        })
    }

    fn check_node(&mut self, node: Positioned<Node>) -> Result<NodeInfo, CheckerError> {
        match node.data {
            Node::Value(_) => self.check_value_node(node),
            Node::FunctionDefinition { .. } => self.check_function_definition(node),
            Node::FunctionCall { .. } => self.check_function_call(node),
            Node::Use(_) => unreachable!("Should have been separated in the IR Generator and should have panicked in the symbolizer!"),
            Node::VariableDefinition { .. } => self.check_variable_definition(node),
        }
    }

    pub fn check(&mut self) -> Result<IROutput, CheckerError> {
        let mut output = IROutput { includes: self.ir_output.includes.clone() , ast: Vec::new() };

        while let Some(node) = self.current() {
            output.ast.push(self.check_node(node)?.checked);
            self.advance();
        }

        Ok(output)
    }

}