use crate::{util::position::Positioned, parser::node::Node};

#[derive(Clone, Debug)]
pub enum IncludeType {
    External,
    StdExternal,
    Internal
}

#[derive(Clone, Debug)]
pub struct Include {
    pub include_type: IncludeType,
    pub path: Positioned<String>
}

#[derive(Clone, Debug)]
pub struct IROutput {
    pub includes: Vec<Include>,
    pub ast: Vec<Positioned<Node>>   
}