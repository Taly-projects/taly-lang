use crate::{ir::output::IROutput, symbolizer::{scope::{Scope, ScopeType}, error::SymbolizerError, trace::Trace}, util::{reference::MutRef, position::{Positioned}}, parser::node::{Node, VarType}};

////////////////////////////////////////////////////////////////////////////////////////////////////
//                                            Symbolizer                                          //
//////////////////////////////////////////////////////////////////////////////////////////////////// 

pub struct Symbolizer {
    ir_output: IROutput,
    index: usize,
    trace: Trace
}

impl Symbolizer {

    pub fn new(ir_output: IROutput) -> Self {
        Self {
            ir_output,
            index: 0,
            trace: Trace::default()
        }
    }

    fn current(&self) -> Option<Positioned<Node>> {
        self.ir_output.ast.get(self.index).cloned()
    }

    fn advance(&mut self) {
        self.index += 1;
    }

    fn symbolize_function_definition(&mut self, node: Positioned<Node>, scope: MutRef<Scope>) -> Result<(), SymbolizerError> {
        let Node::FunctionDefinition { name, external, constructor, parameters, return_type, body, access } = node.data.clone() else {
            unreachable!()
        };

        let function_scope = Scope::new(node.convert(()), ScopeType::Function { 
            name: name.clone(), 
            params: parameters.clone(), 
            children: Vec::new(), 
            return_type, 
            external,
            constructor
        }, Some(scope.clone()), self.trace.clone(), access);

        // Check if unique
        if let Some(previous) = scope.get().enter_function(Trace::full(), name.data.clone(), true, true) {
            return Err(SymbolizerError::SymbolAlreadyDefined(name, previous.get().pos.clone()));
        }
        
        scope.get().add_child(function_scope);

        // Symbolize Params
        let function_scope_ref = scope.get().get_last();
        for param in parameters {
            let param_scope = Scope::new(param.get_position(), ScopeType::Variable { 
                var_type: param.get_position().convert(VarType::Constant), 
                name: param.name.clone(), 
                data_type: Some(param.data_type.clone()), 
                initialized: true 
            }, Some(function_scope_ref.clone()), Trace::default(), None);

            // Check if unique
            if let Some(previous) = scope.get().enter_variable(Trace::full(), param.name.data.clone(), true, false) {
                return Err(SymbolizerError::SymbolAlreadyDefined(param.name, previous.get().pos.clone()));
            }

            function_scope_ref.get().add_child(param_scope);
        }

        // Symbolize children
        self.trace = Trace::new(0, self.trace.clone());
        for node in body {
            self.symbolize_node(node, function_scope_ref.clone())?;
            self.trace.index += 1;
        }
        self.trace = *self.trace.clone().parent.unwrap();

        Ok(())
    }

    fn symbolize_variable_definition(&mut self, node: Positioned<Node>, scope: MutRef<Scope>) -> Result<(), SymbolizerError> {
        let Node::VariableDefinition { var_type, name, data_type, value, access } = node.data.clone() else {
            unreachable!()
        };

        let variable_scope = Scope::new(node.convert(()), ScopeType::Variable { 
            var_type: var_type.clone(), 
            name: name.clone(), 
            data_type: data_type.clone(), 
            initialized: value.is_some() 
        }, Some(scope.clone()), self.trace.clone(), access);

        // Check if unique
        if let Some(previous) = scope.get().enter_variable(Trace::full(), name.data.clone(), true, true) {
            return Err(SymbolizerError::SymbolAlreadyDefined(name, previous.get().pos.clone()));
        }

        scope.get().add_child(variable_scope);

        Ok(())
    }

    fn symbolize_class_definition(&mut self, node: Positioned<Node>, scope: MutRef<Scope>) -> Result<(), SymbolizerError> {
        let Node::ClassDefinition { name, body, access } = node.data.clone() else {
            unreachable!()
        };

        let linked_space = if let Some(class) = scope.get().enter_space(Trace::full(), name.data.clone()) {
            let ScopeType::Space { linked_class, .. } = &mut class.get().scope else {
                unreachable!()
            };

            *linked_class = true;
            true
        } else {
            false
        };

        let class_scope = Scope::new(node.convert(()), ScopeType::Class { 
            name: name.clone(), 
            children: Vec::new(),
            linked_space
        }, Some(scope.clone()), self.trace.clone(), access);

        // Check if unique
        if let Some(previous) = scope.get().enter_class(Trace::full(), name.data.clone()) {
            return Err(SymbolizerError::SymbolAlreadyDefined(name, previous.get().pos.clone()));
        }

        scope.get().add_child(class_scope);

        let class_scope_ref = scope.get().get_last();

        // Symbolize children
        self.trace = Trace::new(0, self.trace.clone());
        for node in body {
            self.symbolize_node(node, class_scope_ref.clone())?;
            self.trace.index += 1;
        }
        self.trace = *self.trace.clone().parent.unwrap();

        // scope.get().add_child(class_scope);

        Ok(())
    }

    fn symbolize_space_definition(&mut self, node: Positioned<Node>, scope: MutRef<Scope>) -> Result<(), SymbolizerError> {
        let Node::SpaceDefinition { name, body, access } = node.data.clone() else {
            unreachable!()
        };

        let linked_class = if let Some(space) = scope.get().enter_class(Trace::full(), name.data.clone()) {
            let ScopeType::Class { linked_space, .. } = &mut space.get().scope else {
                unreachable!()
            };

            *linked_space = true;
            true
        } else {
            false
        };

        let space_scope = Scope::new(node.convert(()), ScopeType::Space { 
            name: name.clone(), 
            children: Vec::new(),
            linked_class
        }, Some(scope.clone()), self.trace.clone(), access);

        // Check if unique
        if let Some(previous) = scope.get().enter_space(Trace::full(), name.data.clone()) {
            return Err(SymbolizerError::SymbolAlreadyDefined(name, previous.get().pos.clone()));
        }

        scope.get().add_child(space_scope);

        let space_scope_ref = scope.get().get_last();

        // Symbolize children
        self.trace = Trace::new(0, self.trace.clone());
        for node in body {
            self.symbolize_node(node, space_scope_ref.clone())?;
            self.trace.index += 1;
        }
        self.trace = *self.trace.clone().parent.unwrap();

        // scope.get().add_child(class_scope);

        Ok(())
    }

    fn symbolize_node(&mut self, node: Positioned<Node>, scope: MutRef<Scope>) -> Result<(), SymbolizerError> {
        match node.data {
            Node::FunctionDefinition { .. } => self.symbolize_function_definition(node, scope),
            Node::Use(_) => unreachable!("Should have been separated in the IR Generator!"),
            Node::VariableDefinition { .. } => self.symbolize_variable_definition(node, scope),
            Node::ClassDefinition { .. } => self.symbolize_class_definition(node, scope),
            Node::SpaceDefinition { .. } => self.symbolize_space_definition(node, scope),
            Node::_Unchecked(inner) => self.symbolize_node(*inner, scope),
            _ => Ok(())
        }
    }

    pub fn symbolize(&mut self, root: MutRef<Scope>) -> Result<(), SymbolizerError> {
        while let Some(current) = self.current() {
            self.symbolize_node(current, root.clone())?;
            self.advance();
            self.trace.index += 1;
        }
        
        Ok(())
    }

}