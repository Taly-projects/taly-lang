use crate::{ir::output::IROutput, symbolizer::{scope::{Scope, ScopeType}, error::SymbolizerError}, util::{reference::MutRef, position::{Positioned}}, parser::node::Node};

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
            params: parameters, 
            children: Vec::new(), 
            return_type, 
            external
        }, Some(scope.clone()));

        // TODO: Symbolize Params

        // Symbolize children
        for node in body {
            self.symbolize_node(node, MutRef::new(&mut function_scope))?;
        }

        // Check if unique
        if let Some(previous) = scope.get().enter_function(name.data.clone()) {
            return Err(SymbolizerError::SymbolAlreadyDefined(name, previous.get().pos.clone()));
        }

        scope.get().add_child(function_scope);

        Ok(())
    }

    fn symbolize_node(&mut self, node: Positioned<Node>, scope: MutRef<Scope>) -> Result<(), SymbolizerError> {
        match node.data {
            Node::Value(_) => Ok(()),
            Node::FunctionDefinition { .. } => self.symbolize_function_definition(node, scope),
            Node::FunctionCall { .. } => Ok(()),
            Node::Use(_) => unreachable!("Should have been separated in the IR Generator!"),
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