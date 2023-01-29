use crate::util::position::Positioned;

////////////////////////////////////////////////////////////////////////////////////////////////////
//                                              Node                                              //
//////////////////////////////////////////////////////////////////////////////////////////////////// 

#[derive(Clone, Debug)]
pub enum Node {
    Value(ValueNode),
    FunctionDefinition {
        name: Positioned<String>,
        external: bool,
        parameters: Vec<FunctionDefinitionParameter>,
        return_type: Option<Positioned<String>>,
        body: Vec<Positioned<Node>>
    },
    FunctionCall {
        name: Positioned<String>,
        parameters: Vec<Positioned<Node>>
    },
    Use(Positioned<String>)
}



////////////////////////////////////////////////////////////////////////////////////////////////////
//                                           Value Node                                           //
//////////////////////////////////////////////////////////////////////////////////////////////////// 

#[derive(Clone, Debug)]
pub enum ValueNode {
    String(String)
}



////////////////////////////////////////////////////////////////////////////////////////////////////
//                                           Value Node                                           //
//////////////////////////////////////////////////////////////////////////////////////////////////// 

#[derive(Clone, Debug)]
pub struct FunctionDefinitionParameter {
    pub name: Positioned<String>,
    pub data_type: Positioned<String>
}

impl FunctionDefinitionParameter {

    pub fn new(name: Positioned<String>, data_type: Positioned<String>) -> Self {
        Self {
            name,
            data_type
        }
    }

}