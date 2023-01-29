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

impl Node {

    pub fn short_name(&self) -> String {
        match self {
            Node::Value(value) => match value {
                ValueNode::String(str) => format!("String({})", str),
            },
            Node::FunctionDefinition { name, .. } => format!("Function({})", name.data),
            Node::FunctionCall { name, .. } => format!("FunctionCall({})", name.data),
            Node::Use(path) => format!("Use({})", path.data),
        }
    }

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