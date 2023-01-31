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
    Use(Positioned<String>),
    VariableDefinition {
        var_type: Positioned<VarType>,
        name: Positioned<String>,
        data_type: Option<Positioned<String>>,
        value: Option<Box<Positioned<Node>>>
    }
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
            Node::VariableDefinition { name, .. } => format!("Variable({})", name.data)
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
//                                  Function Definition Parameter                                 //
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

    pub fn get_position(&self) -> Positioned<()> {
        Positioned::new((), self.name.start.clone(), self.data_type.end.clone())
    }

}



////////////////////////////////////////////////////////////////////////////////////////////////////
//                                          Variable Type                                         //
//////////////////////////////////////////////////////////////////////////////////////////////////// 

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VarType {
    Variable,
    Constant
}