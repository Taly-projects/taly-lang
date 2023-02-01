use crate::{util::{reference::MutRef, position::{Positioned, Position}}, parser::node::{FunctionDefinitionParameter, VarType}, symbolizer::trace::Trace};

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
    },
    Class {
        name: Positioned<String>,
        children: Vec<Scope>
    }
}

#[derive(Clone, Debug)]
pub struct Scope {
    pub pos: Positioned<()>,
    pub scope: ScopeType,
    pub parent: Option<MutRef<Scope>>,
    pub trace: Trace
}

impl Scope {

    pub fn root() -> Self {
        Self {
            pos: Positioned::new((), Position::default(), Position::default()),
            scope: ScopeType::Root { 
                children: Vec::new()
            },
            parent: None,
            trace: Trace::default()
        }
    }

    pub fn new(pos: Positioned<()>, scope: ScopeType, parent: Option<MutRef<Scope>>, trace: Trace) -> Self {
        Self {
            pos,
            scope,
            parent,
            trace
        }
    }

    pub fn add_child(&mut self, scope: Scope) {
        match &mut self.scope {
            ScopeType::Root { children } |
            ScopeType::Function { children, .. } |
            ScopeType::Class { children, .. } => {
                children.push(scope);
            }
            ScopeType::Variable { .. } => {
                panic!("cannot add child here!")
            }
        }
    }

    pub fn get_last(&mut self) -> MutRef<Scope> {
        match &mut self.scope {
            ScopeType::Root { children } |
            ScopeType::Function { children, .. } |
            ScopeType::Class { children, .. } => {
                MutRef::new(children.last_mut().unwrap())
            }
            ScopeType::Variable { .. } => {
                panic!("cannot have children here!")
            }
        }
    }

    pub fn enter_function(&mut self, trace: Trace, name: String) -> Option<MutRef<Scope>> {
        println!("{:#?}", self);
        println!("u");
        match &mut self.scope {
            ScopeType::Root { children } |
            ScopeType::Class {children, ..} => {
                for child in children.iter_mut() {
                    if let ScopeType::Function { name: c_name, .. } = &child.scope {
                        if c_name.data == name && (trace.full || child.trace.index <= trace.index) {
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

    pub fn get_function(&mut self, trace: Trace, name: String) -> Option<MutRef<Scope>> {
        if let Some(fun) = self.enter_function(trace.clone(), name.clone()) {
            return Some(fun);
        }
        if let Some(parent) = &self.parent {
            return parent.get().get_function(*trace.parent.unwrap(), name);
        }
        None
    }

    pub fn enter_variable(&mut self, trace: Trace, name: String) -> Option<MutRef<Scope>> {
        match &mut self.scope {
            ScopeType::Root { children } |
            ScopeType::Function { children, .. } |
            ScopeType::Class { children, .. } => {
                for child in children.iter_mut() {
                    if let ScopeType::Variable { name: c_name, .. } = &child.scope {
                        if c_name.data == name && (trace.full || child.trace.index <= trace.index) {
                            return Some(MutRef::new(child));
                        }
                    }
                }
                None
            },
            ScopeType::Variable { .. } => None,
        }
    }

    pub fn get_variable(&mut self, trace: Trace, name: String) -> Option<MutRef<Scope>> {
        if let Some(fun) = self.enter_variable(trace.clone(), name.clone()) {
            return Some(fun);
        }
        if let Some(parent) = &self.parent {
            return parent.get().get_variable(*trace.parent.unwrap(), name);
        }
        None
    }

    pub fn enter_class(&mut self, trace: Trace, name: String) -> Option<MutRef<Scope>> {
        match &mut self.scope {
            ScopeType::Root { children } |
            ScopeType::Function { children, .. } |
            ScopeType::Class { children, .. } => {
                for child in children.iter_mut() {
                    if let ScopeType::Class { name: c_name, .. } = &child.scope {
                        if c_name.data == name && (trace.full || child.trace.index <= trace.index) {
                            return Some(MutRef::new(child));
                        }
                    }
                }
                None
            },
            ScopeType::Variable { .. } => None,
        }
    }

    pub fn get_class(&mut self, trace: Trace, name: String) -> Option<MutRef<Scope>> {
        if let Some(fun) = self.enter_class(trace.clone(), name.clone()) {
            return Some(fun);
        }
        if let Some(parent) = &self.parent {
            return parent.get().get_class(*trace.parent.unwrap(), name);
        }
        None
    }

}