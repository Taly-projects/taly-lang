use crate::{util::{reference::MutRef, position::{Positioned, Position}}, parser::node::{FunctionDefinitionParameter, VarType}};

////////////////////////////////////////////////////////////////////////////////////////////////////
//                                              Scope                                             //
//////////////////////////////////////////////////////////////////////////////////////////////////// 

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
    },
    Variable {
        var_type: Positioned<VarType>,
        name: Positioned<String>,
        data_type: Option<Positioned<String>>,
        initialized: bool,
    }
}

#[derive(Clone, Debug)]
pub struct Scope {
    pub pos: Positioned<()>,
    pub scope: ScopeType,
    pub parent: Option<MutRef<Scope>>
}

impl Scope {

    pub fn root() -> Self {
        Self {
            pos: Positioned::new((), Position::default(), Position::default()),
            scope: ScopeType::Root { 
                children: Vec::new()
            },
            parent: None
        }
    }

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
            ScopeType::Variable { .. } => {
                panic!("cannot add child here!")
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
            ScopeType::Variable { .. } => None,
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

    pub fn enter_variable(&mut self, name: String) -> Option<MutRef<Scope>> {
        match &mut self.scope {
            ScopeType::Root { children } => {
                for child in children.iter_mut() {
                    if let ScopeType::Variable { name: c_name, .. } = &child.scope {
                        if c_name.data == name {
                            return Some(MutRef::new(child));
                        }
                    }
                }
                None
            },
            ScopeType::Function { children, .. } => {
                for child in children.iter_mut() {
                    if let ScopeType::Variable { name: c_name, .. } = &child.scope {
                        if c_name.data == name {
                            return Some(MutRef::new(child));
                        }
                    }
                }
                None
            },
            ScopeType::Variable { .. } => None,
        }
    }

    pub fn get_variable(&mut self, name: String) -> Option<MutRef<Scope>> {
        if let Some(fun) = self.enter_variable(name.clone()) {
            return Some(fun);
        }
        if let Some(parent) = &self.parent {
            return parent.get().get_variable(name);
        }
        None
    }

}