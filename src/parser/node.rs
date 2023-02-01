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
        constructor: bool,
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
    },
    VariableCall(String),
    BinaryOperation {
        lhs: Box<Positioned<Node>>,
        operator: Positioned<Operator>,
        rhs: Box<Positioned<Node>>
    },
    Return(Option<Box<Positioned<Node>>>),
    ClassDefinition {
        name: Positioned<String>,
        body: Vec<Positioned<Node>>
    },
    SpaceDefinition {
        name: Positioned<String>,
        body: Vec<Positioned<Node>>
    },
    // Compiler Specific Annotation
    _Unchecked(Box<Positioned<Node>>)
}

impl Node {

    pub fn short_name(&self) -> String {
        match self {
            Node::Value(value) => match value {
                ValueNode::String(str) => format!("String({})", str),
                ValueNode::Bool(b) => format!("Bool({})", b),
                ValueNode::Integer(num) => format!("Integer({})", num),
                ValueNode::Decimal(num) => format!("Decimal({})", num),
                ValueNode::Type(str) => format!("Type({})", str),
            },
            Node::FunctionDefinition { name, constructor, .. } => if *constructor {
                    format!("Constructor({})", name.data)
                } else {
                    format!("Function({})", name.data)
                },
            Node::FunctionCall { name, .. } => format!("FunctionCall({})", name.data),
            Node::Use(path) => format!("Use({})", path.data),
            Node::VariableDefinition { name, .. } => format!("Variable({})", name.data),
            Node::VariableCall(name) => format!("VariableCall({})", name),
            Node::BinaryOperation { operator, .. } => match operator.data {
                Operator::Add => format!("BinaryOP(Addition)"),
                Operator::Subtract => format!("BinaryOP(Subtraction)"),
                Operator::Multiply => format!("BinaryOP(Multiplication)"),
                Operator::Divide => format!("BinaryOP(Division)"),
                Operator::Assign => format!("BinaryOP(Assignment)"),
                Operator::Access => format!("BinaryOP(Access)"),
            }
            Node::Return(_) => format!("Return"),
            Node::ClassDefinition { name, .. } => format!("Class({})", name.data),
            Node::SpaceDefinition { name, .. } => format!("Space({})", name.data),
            Node::_Unchecked(inner) => inner.data.short_name(),
        }
    }

}



////////////////////////////////////////////////////////////////////////////////////////////////////
//                                           Value Node                                           //
//////////////////////////////////////////////////////////////////////////////////////////////////// 

#[derive(Clone, Debug)]
pub enum ValueNode {
    String(String),
    Bool(bool),
    Integer(String),
    Decimal(String),
    Type(String)
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



////////////////////////////////////////////////////////////////////////////////////////////////////
//                                            Operator                                            //
//////////////////////////////////////////////////////////////////////////////////////////////////// 

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Assign,
    Access
}