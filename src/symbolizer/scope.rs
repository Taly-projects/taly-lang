use crate::{util::{reference::MutRef, position::{Positioned, Position}}, parser::node::{FunctionDefinitionParameter, VarType, AccessModifier}, symbolizer::trace::Trace};

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
        external: bool,
        constructor: bool
    },
    Variable {
        var_type: Positioned<VarType>,
        name: Positioned<String>,
        data_type: Option<Positioned<String>>,
        initialized: bool,
    },
    Class {
        name: Positioned<String>,
        children: Vec<Scope>,
        linked_space: bool
    },
    Space {
        name: Positioned<String>,
        children: Vec<Scope>,
        linked_class: bool
    },
    Branch {
        children: Vec<Scope>
    }
}

#[derive(Clone, Debug)]
pub struct Scope {
    pub pos: Positioned<()>,
    pub scope: ScopeType,
    pub parent: Option<MutRef<Scope>>,
    pub trace: Trace,
    pub access: Option<Positioned<AccessModifier>>
}

impl Scope {

    pub fn root() -> Self {
        Self {
            pos: Positioned::new((), Position::default(), Position::default()),
            scope: ScopeType::Root { 
                children: Vec::new()
            },
            parent: None,
            trace: Trace::default(),
            access: None
        }
    }

    pub fn new(pos: Positioned<()>, scope: ScopeType, parent: Option<MutRef<Scope>>, trace: Trace, access: Option<Positioned<AccessModifier>>) -> Self {
        Self {
            pos,
            scope,
            parent,
            trace,
            access
        }
    }

    pub fn short_name(&self) -> String {
        match &self.scope {
            ScopeType::Root { .. } => "Root".to_string(),
            ScopeType::Function { name, .. } => format!("Function({})", name.data),
            ScopeType::Variable { name, data_type, .. } => format!("Variable({:?}, {})", data_type, name.data),
            ScopeType::Class { name, .. } => format!("Class({})", name.data),
            ScopeType::Space { name, .. } => format!("Space({})", name.data),
            ScopeType::Branch { .. } => "Branch".to_string()
        }
    }

    pub fn process_name(&mut self) -> String {
        let mut buf = String::new();
        if let Some(parent) = &self.parent {
            buf.push_str(&parent.get().process_name());
            if !self.is_root() && !parent.get().is_root() && !self.is_branch() && !parent.get().is_branch() {
                buf.push('_');
            }
        } 
        match &self.scope {
            ScopeType::Function { name, .. } => buf.push_str(&name.data),
            ScopeType::Variable { name, .. } => return name.data.clone(),
            ScopeType::Class { name, .. } => buf.push_str(&name.data),
            ScopeType::Space { name, .. } => buf.push_str(&name.data),
            _ => {}
        }

        buf
    }

    pub fn is_root(&self) -> bool {
        match self.scope {
            ScopeType::Root { .. } => true,
            _ => false
        }
    }

    pub fn is_variable(&self) -> bool {
        match self.scope {
            ScopeType::Variable { .. } => true,
            _ => false
        }
    }

    pub fn is_branch(&self) -> bool {
        match self.scope {
            ScopeType::Branch { .. } => true,
            _ => false
        }
    }

    pub fn add_child(&mut self, scope: Scope) {
        match &mut self.scope {
            ScopeType::Root { children } |
            ScopeType::Function { children, .. } |
            ScopeType::Class { children, .. } |
            ScopeType::Space { children, .. } |
            ScopeType::Branch { children } => {
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
            ScopeType::Class { children, .. } |
            ScopeType::Space { children, .. } |
            ScopeType::Branch { children } => {
                MutRef::new(children.last_mut().unwrap())
            }
            ScopeType::Variable { .. } => {
                panic!("cannot have children here!")
            }
        }
    }

    pub fn get_child(&mut self, index: usize) -> MutRef<Scope> {
        match &mut self.scope {
            ScopeType::Root { children } |
            ScopeType::Function { children, .. } |
            ScopeType::Class { children, .. } |
            ScopeType::Space { children, .. } |
            ScopeType::Branch { children } => {
                MutRef::new(children.get_mut(index).unwrap())
            }
            ScopeType::Variable { .. } => {
                panic!("cannot have children here!")
            }
        }
    } 

    fn get_function_in_children(children: &mut Vec<Scope>, trace: Trace, name: String) -> Option<MutRef<Scope>> {
        for child in children.iter_mut() {
            if let ScopeType::Function { name: var_name, .. } = &child.scope {
                if var_name.data == name && (trace.full || child.trace.index <= trace.index) {
                    return Some(MutRef::new(child));
                }
            }
        }
        None
    }

    fn get_constructor_in_children(children: &mut Vec<Scope>, trace: Trace, name: String) -> Option<MutRef<Scope>> {
        for child in children.iter_mut() {
            if let ScopeType::Function { name: var_name, constructor, .. } = &child.scope {
                if *constructor && var_name.data == name && (trace.full || child.trace.index <= trace.index) {
                    return Some(MutRef::new(child));
                }
            }
        }
        None
    }

    pub fn enter_function(&mut self, trace: Trace, name: String, look_links: bool, allow_fields: bool) -> Option<MutRef<Scope>> {
        let space_to_check;

        match &mut self.scope {
            ScopeType::Root { children } => return Self::get_function_in_children(children, trace, name),
            ScopeType::Variable { data_type, .. } => {
                if let Some(data_type) = data_type.clone() {
                    println!("Looking for '{}'", data_type.data);
                    if let Some(class) = self.get_class(trace.clone(), data_type.data.clone()) {
                        return class.get().enter_function(trace, name, look_links, true);
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            },
            ScopeType::Class { name: class_name, children, linked_space, .. } => {
                if allow_fields {
                    return Self::get_function_in_children(children, trace, name);
                } 
                if let Some(constructor) = Self::get_constructor_in_children(children, trace.clone(), name.clone()) {
                    return Some(constructor);
                }
                if linked_space.clone() && look_links {
                    space_to_check = Some(class_name.clone());                        
                } else {
                    return None;
                }
            },
            ScopeType::Space { children, .. } => return Self::get_function_in_children(children, trace, name),
            _ => return None
        }

        if let Some(linked_space) = space_to_check {
            if let Some(space) = self.get_space(trace.clone(), linked_space.data.clone()) {
                space.get().enter_function(trace, name, false, false)
            } else {
                None
            }
        } else {
            None
        }
    }
    
    pub fn get_function(&mut self, trace: Trace, name: String, allow_fields: bool) -> Option<MutRef<Scope>> {
        if let Some(fun) = self.enter_function(trace.clone(), name.clone(), true, allow_fields) {
            return Some(fun);
        }
        if let Some(parent) = &self.parent {
            if trace.full {
                return parent.get().get_function(trace, name, allow_fields);
            } else {
                return parent.get().get_function(*trace.parent.unwrap(), name, allow_fields);
            }
        }
        None
    }

    fn get_variable_in_children(children: &mut Vec<Scope>, trace: Trace, name: String) -> Option<MutRef<Scope>> {
        for child in children.iter_mut() {
            if let ScopeType::Variable { name: var_name, .. } = &child.scope {
                if var_name.data == name && (trace.full || child.trace.index <= trace.index) {
                    return Some(MutRef::new(child));
                }
            }
        }
        None
    }

    pub fn enter_variable(&mut self, trace: Trace, name: String, look_links: bool, allow_fields: bool) -> Option<MutRef<Scope>> {
        match &mut self.scope {
            ScopeType::Function { children, .. } => Self::get_variable_in_children(children, trace, name),
            ScopeType::Variable { data_type, .. } => {
                if let Some(data_type) = data_type.clone() {
                    if let Some(class) = self.get_class(trace.clone(), data_type.data.clone()) {
                        class.get().enter_variable(trace, name, look_links, true)
                    } else {
                        None
                    }
                } else {
                    None
                }
            },
            ScopeType::Class { children, .. } if allow_fields => Self::get_variable_in_children(children, trace, name),
            ScopeType::Branch { children} => Self::get_variable_in_children(children, trace, name),
            _ => None
        }
    }

    pub fn get_variable(&mut self, trace: Trace, name: String, allow_fields: bool) -> Option<MutRef<Scope>> {
        if let Some(fun) = self.enter_variable(trace.clone(), name.clone(), true, allow_fields) {
            return Some(fun);
        }
        if let Some(parent) = &self.parent {
            if trace.full {
                return parent.get().get_variable(trace, name, allow_fields);
            } else {
                return parent.get().get_variable(*trace.parent.unwrap(), name, allow_fields);
            }
        }
        None
    }

    pub fn enter_class(&mut self, trace: Trace, name: String) -> Option<MutRef<Scope>> {
        match &mut self.scope {
            ScopeType::Root { children } |
            ScopeType::Function { children, .. } |
            ScopeType::Class { children, .. } |
            ScopeType::Space { children, .. } => {
                for child in children.iter_mut() {
                    if let ScopeType::Class { name: c_name, .. } = &child.scope {
                        if c_name.data == name && (trace.full || child.trace.index <= trace.index) {
                            return Some(MutRef::new(child));
                        }
                    }
                }
                None
            },
            _ => None,
        }
    }

    pub fn get_class(&mut self, trace: Trace, name: String) -> Option<MutRef<Scope>> {
        if let Some(fun) = self.enter_class(trace.clone(), name.clone()) {
            return Some(fun);
        }
        if let Some(parent) = &self.parent {
            println!("Looking for {}, Parent {:#?}", name, parent.get().short_name());
            if trace.full {
                return parent.get().get_class(trace, name);
            } else {
                return parent.get().get_class(*trace.parent.unwrap(), name);
            }
        }
        None
    }

    pub fn enter_space(&mut self, trace: Trace, name: String) -> Option<MutRef<Scope>> {
        match &mut self.scope {
            ScopeType::Root { children } |
            ScopeType::Function { children, .. } |
            ScopeType::Class { children, .. } |
            ScopeType::Space { children, .. } => {
                for child in children.iter_mut() {
                    if let ScopeType::Space { name: c_name, .. } = &child.scope {
                        if c_name.data == name && (trace.full || child.trace.index <= trace.index) {
                            return Some(MutRef::new(child));
                        }
                    }
                }
                None
            },
            _ => None,
        }
    }

    pub fn get_space(&mut self, trace: Trace, name: String) -> Option<MutRef<Scope>> {
        if let Some(fun) = self.enter_space(trace.clone(), name.clone()) {
            return Some(fun);
        }
        if let Some(parent) = &self.parent {
            if trace.full {
                return parent.get().get_space(trace, name);
            } else {
                return parent.get().get_space(*trace.parent.unwrap(), name);
            }
        }
        None
    }

}