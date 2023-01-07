use crate::util::position::Positioned;

////////////////////////////////////////////////////////////////////////////////////////////////////
//                                              Node                                              //
//////////////////////////////////////////////////////////////////////////////////////////////////// 

#[derive(Clone, Debug)]
pub enum Node {
    Value(ValueNode),
    Use(Positioned<String>)
}



////////////////////////////////////////////////////////////////////////////////////////////////////
//                                           Value Node                                           //
//////////////////////////////////////////////////////////////////////////////////////////////////// 

#[derive(Clone, Debug)]
pub enum ValueNode {
    String(String)
}