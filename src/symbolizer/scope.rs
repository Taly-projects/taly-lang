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
    pub pos: Positioned<()>,
    pub scope: ScopeType,
    pub parent: Option<MutRef<Scope>>
}

impl Scope {

    pub fn new(pos: Positioned<()>, scope: ScopeType, parent: Option<MutRef<Scope>>) -> Self {
        Self {
            pos,
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

    pub fn enter_function(&mut self, name: String) -> Option<MutRef<Scope>> {
        match &mut self.scope {
            ScopeType::Root { children } => {
                for child in children.iter_mut() {
                    if let ScopeType::Function { name: c_name, .. } = &child.scope {
                        if c_name.data == name {
                            return Some(MutRef::new(child));
                        }
                    }
                }
                None
            },
            ScopeType::Function { .. } => None,
        }
    }

    pub fn get_function(&mut self, name: String) -> Option<MutRef<Scope>> {
        if let Some(fun) = self.enter_function(name.clone()) {
            return Some(fun);
        }
        if let Some(parent) = &self.parent {
            return parent.get().get_function(name);
        }
        None
    }

}