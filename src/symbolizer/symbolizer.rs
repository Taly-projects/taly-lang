use crate::{ir::output::IROutput, symbolizer::{scope::{Scope, ScopeType}, error::SymbolizerError}, util::{reference::MutRef, position::{Positioned}}, parser::node::{Node, VarType}};

////////////////////////////////////////////////////////////////////////////////////////////////////
//                                            Symbolizer                                          //
//////////////////////////////////////////////////////////////////////////////////////////////////// 

pub struct Symbolizer {
    ir_output: IROutput,
    index: usize,
}

impl Symbolizer {

    pub fn new(ir_output: IROutput) -> Self {
        Self {
            ir_output,
            index: 0, 
        }
    }

    fn current(&self) -> Option<Positioned<Node>> {
        self.ir_output.ast.get(self.index).cloned()
    }

    fn advance(&mut self) {
        self.index += 1;
    }

    fn symbolize_function_definition(&mut self, node: Positioned<Node>, scope: MutRef<Scope>) -> Result<(), SymbolizerError> {
        let Node::FunctionDefinition { name, external, parameters, return_type, body } = node.data.clone() else {
            unreachable!()
        };

        let mut function_scope = Scope::new(node.convert(()), ScopeType::Function { 
            name: name.clone(), 
            params: parameters.clone(), 
            children: Vec::new(), 
            return_type, 
            external
        }, Some(scope.clone()));

        // Symbolize Params
        let function_scope_ref = MutRef::new(&mut function_scope);
        for param in parameters {
            let param_scope = Scope::new(param.get_position(), ScopeType::Variable { 
                var_type: param.get_position().convert(VarType::Constant), 
                name: param.name.clone(), 
                data_type: Some(param.data_type.clone()), 
                initialized: true 
            }, Some(function_scope_ref.clone()));

            // Check if unique
            if let Some(previous) = scope.get().enter_variable(param.name.data.clone()) {
                return Err(SymbolizerError::SymbolAlreadyDefined(param.name, previous.get().pos.clone()));
            }

            function_scope.add_child(param_scope);
        }

        // Symbolize children
        for node in body {
            self.symbolize_node(node, function_scope_ref.clone())?;
        }

        // Check if unique
        if let Some(previous) = scope.get().enter_function(name.data.clone()) {
            return Err(SymbolizerError::SymbolAlreadyDefined(name, previous.get().pos.clone()));
        }

        scope.get().add_child(function_scope);

        Ok(())
    }

    fn symbolize_variable_definition(&mut self, node: Positioned<Node>, scope: MutRef<Scope>) -> Result<(), SymbolizerError> {
        let Node::VariableDefinition { var_type, name, data_type, value } = node.data.clone() else {
            unreachable!()
        };

        let variable_scope = Scope::new(node.convert(()), ScopeType::Variable { 
            var_type: var_type.clone(), 
            name: name.clone(), 
            data_type: data_type.clone(), 
            initialized: value.is_some() 
        }, Some(scope.clone()));

        // Check if unique
        if let Some(previous) = scope.get().enter_variable(name.data.clone()) {
            return Err(SymbolizerError::SymbolAlreadyDefined(name, previous.get().pos.clone()));
        }

        scope.get().add_child(variable_scope);

        Ok(())
    }

    fn symbolize_node(&mut self, node: Positioned<Node>, scope: MutRef<Scope>) -> Result<(), SymbolizerError> {
        match node.data {
            Node::FunctionDefinition { .. } => self.symbolize_function_definition(node, scope),
            Node::Use(_) => unreachable!("Should have been separated in the IR Generator!"),
            Node::VariableDefinition { .. } => self.symbolize_variable_definition(node, scope),
            _ => Ok(())
        }
    }

    pub fn symbolize(&mut self, root: MutRef<Scope>) -> Result<(), SymbolizerError> {
        while let Some(current) = self.current() {
            self.symbolize_node(current, root.clone())?;
            self.advance();
        }
        
        Ok(())
    }

}