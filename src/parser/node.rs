use crate::util::position::Positioned;

#[derive(Clone, Debug)]
pub enum Node {
    Use(Positioned<String>)
}