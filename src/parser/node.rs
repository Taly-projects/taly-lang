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
        return_type: Option<Positioned<DataType>>,
        body: Vec<Positioned<Node>>,
        access: Option<Positioned<AccessModifier>>
    },
    FunctionCall {
        name: Positioned<String>,
        parameters: Vec<Positioned<Node>>
    },
    Use(Positioned<String>),
    VariableDefinition {
        var_type: Positioned<VarType>,
        name: Positioned<String>,
        data_type: Option<Positioned<DataType>>,
        value: Option<Box<Positioned<Node>>>,
        access: Option<Positioned<AccessModifier>>
    },
    VariableCall(String),
    BinaryOperation {
        lhs: Box<Positioned<Node>>,
        operator: Positioned<Operator>,
        rhs: Box<Positioned<Node>>
    },
    UnaryOperation {
        operator: Positioned<Operator>,
        value: Box<Positioned<Node>>
    },
    Return(Option<Box<Positioned<Node>>>),
    ClassDefinition {
        name: Positioned<String>,
        body: Vec<Positioned<Node>>,
        access: Option<Positioned<AccessModifier>>,
        extensions: Vec<Positioned<String>>
    },
    SpaceDefinition {
        name: Positioned<String>,
        body: Vec<Positioned<Node>>,
        access: Option<Positioned<AccessModifier>>
    },
    IfStatement {
        condition: Box<Positioned<Node>>,
        body: Vec<Positioned<Node>>,
        elif_branches: Vec<ElifBranch>,
        else_body: Vec<Positioned<Node>>
    },
    WhileLoop {
        condition: Box<Positioned<Node>>,
        body: Vec<Positioned<Node>>
    },
    MatchStatement {
        expr: Box<Positioned<Node>>,
        branches: Vec<MatchBranch>,
        else_body: Vec<Positioned<Node>>
    },
    Break(Option<Positioned<String>>),
    Continue(Option<Positioned<String>>),
    Label {
        name: Positioned<String>,
        inner: Box<Positioned<Node>>
    },
    InterfaceDefinition {
        name: Positioned<String>,
        body: Vec<Positioned<Node>>,
        access: Option<Positioned<AccessModifier>>
    },
    // Compiler Specific Annotation
    _Unchecked(Box<Positioned<Node>>),
    _Optional(Box<Positioned<Node>>),
    _Renamed {
        name: String,
        node: Box<Positioned<Node>>
    },
    _Implementation(Box<Positioned<Node>>),
    _Generated(Box<Positioned<Node>>),
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
                Operator::BooleanAnd => format!("BinaryOP(BooleanAnd)"),
                Operator::BooleanOr => format!("BinaryOP(BooleanOr)"),
                Operator::BooleanXor => format!("BinaryOP(BooleanXor)"),
                Operator::Equal => format!("BinaryOP(Equal)"),
                Operator::NotEqual => format!("BinaryOP(NotEqual)"),
                Operator::Greater => format!("BinaryOP(Greater)"),
                Operator::GreaterOrEqual => format!("BinaryOP(GreaterOREqual)"),
                Operator::Less => format!("BinaryOP(Less)"),
                Operator::LessOrEqual => format!("BinaryOP(LessOREqual)"),
                _ => unreachable!()
            }
            Node::UnaryOperation { operator, .. } => match operator.data {
                Operator::Add => format!("UnaryOP(Positive)"),
                Operator::Subtract => format!("UnaryOP(Negative)"),
                Operator::BooleanNot => format!("UnaryOP(Boolean Negative)"),
                _ => unreachable!()
            }
            Node::Return(_) => format!("Return"),
            Node::ClassDefinition { name, .. } => format!("Class({})", name.data),
            Node::SpaceDefinition { name, .. } => format!("Space({})", name.data),
            Node::IfStatement { .. } => format!("If"),
            Node::WhileLoop { .. } => format!("While"),
            Node::MatchStatement { .. } => format!("Match"),
            Node::Break(_) => format!("break"),
            Node::Continue(_) => format!("continue"),
            Node::Label { name, .. } => format!("Label({})", name.data),
            Node::InterfaceDefinition { name, .. } => format!("Interface({})", name.data),
            Node::_Unchecked(inner) => format!("!{}", inner.data.short_name()),
            Node::_Optional(inner) => format!("?{}", inner.data.short_name()),
            Node::_Renamed { node, .. } => format!("*{}", node.data.short_name()),
            Node::_Implementation(inner) => format!("@override {}", inner.data.short_name()),
            Node::_Generated(inner) => format!("@generated {}", inner.data.short_name()),
        }
    }

    pub fn is_generated(&self) -> bool {
        match self {
            Node::_Generated(_) => true,
            _ => false
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
//                                          Variable Type                                         //
//////////////////////////////////////////////////////////////////////////////////////////////////// 

#[derive(Clone, Debug)]
pub enum DataType {
    Custom(String),
    Function {
        return_type: Option<Box<Positioned<DataType>>>,
        params: Vec<Positioned<DataType>>
    }
}

impl ToString for DataType {

    fn to_string(&self) -> String {
        match self {
            DataType::Custom(inner) => inner.clone(),
            DataType::Function { return_type, params } => {
                let mut buf = String::new();
                buf.push_str("fn(");
                let mut first = false;
                for param in params {
                    if !first {
                        buf.push_str(", ");
                    }
                    buf.push_str(&param.data.to_string());
                    first = false;
                }
                buf.push(')');
                if let Some(return_type) = return_type {
                    buf.push_str(": ");
                    buf.push_str(&return_type.data.to_string());
                }
                buf
            },
        }
    }

}

impl PartialEq for DataType {
    
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Custom(l0), Self::Custom(r0)) => l0 == r0,
            (Self::Function { return_type: l_return_type, params: l_params }, Self::Function { return_type: r_return_type, params: r_params }) => {
                match (l_return_type, r_return_type) {
                    (None, None) => {}
                    (Some(l), Some(r)) if l.data == r.data => {},
                    (_, _) => return false
                }
                
                'A: for l_param in l_params.iter() {
                    for r_param in r_params.iter() {
                        if l_param.data == r_param.data {
                            continue 'A;
                        }
                    }
                    return false;
                }
                true
            }
            _ => false,
        }
    }

}

impl Eq for DataType {
    
}



////////////////////////////////////////////////////////////////////////////////////////////////////
//                                  Function Definition Parameter                                 //
//////////////////////////////////////////////////////////////////////////////////////////////////// 

#[derive(Clone, Debug)]
pub struct FunctionDefinitionParameter {
    pub name: Positioned<String>,
    pub data_type: Positioned<DataType>
}

impl FunctionDefinitionParameter {

    pub fn new(name: Positioned<String>, data_type: Positioned<DataType>) -> Self {
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
    Access,
    BooleanAnd,
    BooleanOr,
    BooleanXor,
    BooleanNot,
    Equal,
    NotEqual,
    Greater,
    GreaterOrEqual,
    Less,
    LessOrEqual
}



////////////////////////////////////////////////////////////////////////////////////////////////////
//                                         Access Modifier                                        //
//////////////////////////////////////////////////////////////////////////////////////////////////// 

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AccessModifier {
    Public,
    Private,
    Protected,
    Locked,
    Guarded
}



////////////////////////////////////////////////////////////////////////////////////////////////////
//                                           Elif Branch                                          //
//////////////////////////////////////////////////////////////////////////////////////////////////// 

#[derive(Clone, Debug)]
pub struct ElifBranch {
    pub condition: Positioned<Node>,
    pub body: Vec<Positioned<Node>>
}



////////////////////////////////////////////////////////////////////////////////////////////////////
//                                           Match Branch                                         //
//////////////////////////////////////////////////////////////////////////////////////////////////// 

#[derive(Clone, Debug)]
pub struct MatchBranch {
    pub conditions: Vec<Positioned<Node>>,
    pub body: Vec<Positioned<Node>>
}