use crate::{util::{reference::MutRef, position::Positioned}, parser::node::FunctionDefinitionParameter};

#[derive(Clone, Debug)]
pub enum ScopeType {
    Root {
        children: Vec<Scope>,
    },
    Function {
        name: Positioned<String>,
        params: Vec<FunctionDefinitionParameter>,
        children: Vec<Scope>,
        return_type: Option<Positioned<String>>,
        external: bool
    }
}

#[derive(Clone, Debug)]
pub struct Scope {
    pub scope: ScopeType,
    pub parent: Option<MutRef<Scope>>
}

impl Scope {

    pub fn new(scope: ScopeType, parent: Option<MutRef<Scope>>) -> Self {
        Self {
            scope,
            parent
        }
    }

    pub fn add_child(&mut self, scope: Scope) {
        match &mut self.scope {
            ScopeType::Root { children } => {
                children.push(scope);
            },
            ScopeType::Function { children, .. } => {
                children.push(scope);
            }
        }
    }

}