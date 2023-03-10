use crate::{symbolizer::{scope::{Scope, ScopeType, Scoped}, error::SymbolizerError, trace::Trace}, util::{reference::MutRef, position::{Positioned}}, parser::node::{Node, VarType, AccessModifier, DataType}};

////////////////////////////////////////////////////////////////////////////////////////////////////
//                                            Symbolizer                                          //
//////////////////////////////////////////////////////////////////////////////////////////////////// 

pub struct Symbolizer {
    ast: Vec<Positioned<Node>>,
    index: usize,
    trace: Trace,
}

impl Symbolizer {

    pub fn new(ast: Vec<Positioned<Node>>) -> Self {
        Self {
            ast,
            index: 0,
            trace: Trace::default()
        }
    }

    /* Cursor Movement */
    fn current(&self) -> Option<Positioned<Node>> {
        self.ast.get(self.index).cloned()
    }

    fn advance(&mut self) {
        self.index += 1;
    }

    /* Symbolize */
    fn symbolize_function_definition(&mut self, node: Positioned<Node>, scope: MutRef<Scope>) -> Result<(), SymbolizerError> {
        let Node::FunctionDefinition { name, external, constructor, parameters, return_type, body, access } = node.data.clone() else {
            unreachable!()
        };

        let return_type_scoped = if let Some(return_type) = return_type {
            Some(Scoped {
                data: return_type.clone(), 
                scope: match return_type.data {
                    DataType::Custom(custom) => if let Some(return_type_scope) = scope.get().get_class(Trace::full(), custom) {
                        Some(return_type_scope.clone())
                    } else {
                        None
                    },
                    DataType::Function { .. } => None,
                }
                }
            )
        } else {
            None
        };

        let function_scope = Scope::new(node.convert(()), ScopeType::Function { 
            name: name.clone(), 
            params: parameters.clone(), 
            children: Vec::new(), 
            return_type: return_type_scoped, 
            external,
            constructor,
            implementation: false
        }, Some(scope.clone()), self.trace.clone(), access);

        // Check if unique
        if let Some(previous) = scope.get().enter_function(Trace::full(), name.data.clone(), true, true) {
            return Err(SymbolizerError::SymbolAlreadyDefined(name, previous.get().pos.clone()));
        }
        
        scope.get().add_child(function_scope);

        // Symbolize Params
        let function_scope_ref = scope.get().get_last();
        for param in parameters {
            let param_type_scoped = Some(Scoped {
                data: param.data_type.clone(), 
                scope: match param.data_type.data.clone() {
                    DataType::Custom(inner) => if let Some(param_type_scope) = scope.get().get_class(Trace::full(), inner) {
                        Some(param_type_scope.clone())
                    } else {
                        None
                    },
                    DataType::Function { .. } => None
                }
            });

            let param_scope = Scope::new(param.get_position(), ScopeType::Variable { 
                var_type: param.get_position().convert(VarType::Constant), 
                name: param.name.clone(), 
                data_type: param_type_scoped, 
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

        let data_type_scoped = if let Some(data_type) = data_type {
            Some(Scoped {
                data: data_type.clone(), 
                scope: match data_type.data {
                    DataType::Custom(inner) => if let Some(data_type_scope) = scope.get().get_class(Trace::full(), inner) {
                        Some(data_type_scope.clone())
                    } else {
                        None
                    },
                    DataType::Function { .. } => None
                }
            })
        } else {
            None
        };

        let variable_scope = Scope::new(node.convert(()), ScopeType::Variable { 
            var_type: var_type.clone(), 
            name: name.clone(), 
            data_type: data_type_scoped, 
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
        let Node::ClassDefinition { name, body, access, extensions } = node.data.clone() else {
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

        // Process extensions
        let mut extensions_scope = Vec::new();
        for extension in extensions {
            if let Some(interface_scope) = scope.get().get_interface(Trace::full(), extension.data.clone()) {
                extensions_scope.push(interface_scope.clone());
            } else {
                return Err(SymbolizerError::SymbolNotFound(extension));
            }
        }

        let class_scope = Scope::new(node.convert(()), ScopeType::Class { 
            name: name.clone(), 
            children: Vec::new(),
            linked_space,
            extensions: extensions_scope
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

    fn symbolize_if_statement(&mut self, node: Positioned<Node>, scope: MutRef<Scope>) -> Result<(), SymbolizerError> {
        let Node::IfStatement { body, elif_branches, else_body, .. } = node.data.clone() else {
            unreachable!()
        };

        // Symbolize If
        let if_scope = Scope::new(node.convert(()), ScopeType::Branch { 
            label: None,
            debug_name: "If".to_string(),
            children: Vec::new() 
        }, Some(scope.clone()), self.trace.clone(), Some(node.convert(AccessModifier::Public)));
        
        scope.get().add_child(if_scope);

        let if_scope_ref = scope.get().get_last();

        self.trace = Trace::new(0, self.trace.clone());
        for node in body {
            self.symbolize_node(node, if_scope_ref.clone())?;
            self.trace.index += 1;
        }
        self.trace = *self.trace.clone().parent.unwrap();
        self.trace.index += 1;

        // Symbolize Elif
        for elif_branch in elif_branches {
            let elif_scope = Scope::new(node.convert(()), ScopeType::Branch { 
                label: None,
                debug_name: "Elif".to_string(),
                children: Vec::new() 
            }, Some(scope.clone()), self.trace.clone(), Some(node.convert(AccessModifier::Public)));
        
            scope.get().add_child(elif_scope);
    
            let elif_scope_ref = scope.get().get_last();
            
            self.trace = Trace::new(0, self.trace.clone());
            for node in elif_branch.body {
                self.symbolize_node(node, elif_scope_ref.clone())?;
                self.trace.index += 1;
            }
            self.trace = *self.trace.clone().parent.unwrap();
            self.trace.index += 1;
        }

        // Symbolize Else
        if !else_body.is_empty() {
            let else_scope = Scope::new(node.convert(()), ScopeType::Branch {
                label: None,
                debug_name: "Else".to_string(), 
                children: Vec::new() 
            }, Some(scope.clone()), self.trace.clone(), Some(node.convert(AccessModifier::Public)));
            
            scope.get().add_child(else_scope);
    
            let else_scope_ref = scope.get().get_last();
    
            self.trace = Trace::new(0, self.trace.clone());
            for node in else_body {
                self.symbolize_node(node, else_scope_ref.clone())?;
                self.trace.index += 1;
            }
            self.trace = *self.trace.clone().parent.unwrap();
        }

        Ok(())
    }

    fn symbolize_while_loop(&mut self, node: Positioned<Node>, scope: MutRef<Scope>) -> Result<(), SymbolizerError> {
        let Node::WhileLoop { body, .. } = node.data.clone() else {
            unreachable!()
        };

        // Symbolize If
        let while_scope = Scope::new(node.convert(()), ScopeType::Branch { 
            label: None,
            debug_name: "While".to_string(),
            children: Vec::new() 
        }, Some(scope.clone()), self.trace.clone(), Some(node.convert(AccessModifier::Public)));
        
        scope.get().add_child(while_scope);

        let while_scope_ref = scope.get().get_last();

        self.trace = Trace::new(0, self.trace.clone());
        for node in body {
            self.symbolize_node(node, while_scope_ref.clone())?;
            self.trace.index += 1;
        }
        self.trace = *self.trace.clone().parent.unwrap();
        self.trace.index += 1;

        Ok(())
    }

    fn symbolize_label(&mut self, node: Positioned<Node>, scope: MutRef<Scope>) -> Result<(), SymbolizerError> {
        let Node::Label { name, inner } = node.data.clone() else {
            unreachable!()
        };

        self.symbolize_node(*inner, scope.clone())?;

        let scope_ref = scope.get();
        let last_scope = scope_ref.get_last();
        let ScopeType::Branch { label, .. } = &mut last_scope.get().scope else {
            unreachable!("There should be a branch inside a label")
        };

        *label = Some(name);

        Ok(())
    }

    fn symbolize_interface_definition(&mut self, node: Positioned<Node>, scope: MutRef<Scope>) -> Result<(), SymbolizerError> {
        let Node::InterfaceDefinition { name, body, access } = node.data.clone() else {
            unreachable!()
        };

        let interface_scope = Scope::new(node.convert(()), ScopeType::Interface { 
            name: name.clone(), 
            children: Vec::new()
        }, Some(scope.clone()), self.trace.clone(), access);

        // Check if unique
        if let Some(previous) = scope.get().enter_interface(Trace::full(), name.data.clone()) {
            return Err(SymbolizerError::SymbolAlreadyDefined(name, previous.get().pos.clone()));
        }

        scope.get().add_child(interface_scope);

        let interface_scope_ref = scope.get().get_last();

        // Symbolize children
        self.trace = Trace::new(0, self.trace.clone());
        for node in body {
            self.symbolize_node(node, interface_scope_ref.clone())?;
            self.trace.index += 1;
        }
        self.trace = *self.trace.clone().parent.unwrap();

        Ok(())
    }

    fn symbolize_node(&mut self, node: Positioned<Node>, scope: MutRef<Scope>) -> Result<(), SymbolizerError> {
        match node.data {
            Node::FunctionDefinition { .. } => self.symbolize_function_definition(node, scope),
            Node::Use(_) => Ok(()), // Ignored (will be moved out by the IR Generator) 
            Node::VariableDefinition { .. } => self.symbolize_variable_definition(node, scope),
            Node::ClassDefinition { .. } => self.symbolize_class_definition(node, scope),
            Node::SpaceDefinition { .. } => self.symbolize_space_definition(node, scope),
            Node::IfStatement { .. } => self.symbolize_if_statement(node, scope),
            Node::WhileLoop { .. } => self.symbolize_while_loop(node, scope),
            Node::Label { .. } => self.symbolize_label(node, scope),
            Node::InterfaceDefinition { .. } => self.symbolize_interface_definition(node, scope),
            _ => Ok(())
        }
    }

    pub fn symbolize(&mut self, root: MutRef<Scope>) -> Result<(), SymbolizerError> {
        while let Some(current) = self.current() {
            self.symbolize_node(current.clone(), root.clone())?;
            self.advance();
            if let Node::Use(_) = current.data {
                // Don't advance (ignored)
            } else {
                self.trace.index += 1;
            }
        }
        
        Ok(())
    }

}