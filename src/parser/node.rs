use crate::util::position::Positioned;

#[derive(Clone, Debug)]
pub enum Node {
    Value(ValueNode),
    Use(Positioned<String>)
}

#[derive(Clone, Debug)]
pub enum ValueNode {
    String(String)
}