use crate::{util::position::Positioned, ir::{error::IRError, output::{IROutput, Include, IncludeType}}, parser::node::{Node, ValueNode}};

////////////////////////////////////////////////////////////////////////////////////////////////////
//                                           IR Generator                                         //
//////////////////////////////////////////////////////////////////////////////////////////////////// 

pub struct IRGenerator {
    ast: Vec<Positioned<Node>>,
    index: usize
}

impl IRGenerator {

    pub fn new(ast: Vec<Positioned<Node>>) -> Self {
        Self {
            ast,
            index: 0
        }
    }

    fn advance(&mut self) {
        self.index += 1;
    }

    fn current(&self) -> Option<Positioned<Node>> {
        self.ast.get(self.index).cloned()
    }

    fn generate_include(&mut self, path: Positioned<String>) -> Result<Include, IRError> {
        if path.data.starts_with("c-") {
            Ok(Include { 
                include_type: IncludeType::External, 
                path: path.convert(format!("{}.h", &path.data[2..path.data.len()]))
            })
        } else if path.data.starts_with("std-") {
            Ok(Include { 
                include_type: IncludeType::StdExternal, 
                path: path.convert(format!("{}.h", &path.data[4..path.data.len()]))
            })
        } else {
            Ok(Include { 
                include_type: IncludeType::Internal, 
                path: path.clone() 
            })
        }
    }

    fn generate_value(&mut self, node: Positioned<Node>) -> Result<Positioned<Node>, IRError> {
        let Node::Value(value) = node.data.clone() else {
            unreachable!()
        };

        match value {
            ValueNode::String(str) => Ok(node.convert(Node::Value(ValueNode::String(str.clone())))),
        }
    }

    fn generate_function_call(&mut self, node: Positioned<Node>) -> Result<Positioned<Node>, IRError> {
        let Node::FunctionCall { name, parameters } = node.data.clone() else {
            unreachable!()
        };

        Ok(node.convert(Node::FunctionCall { 
            name: name.clone(), 
            parameters: parameters.clone()
        }))
    }

    fn generate_function_definition(&mut self, node: Positioned<Node>) -> Result<Positioned<Node>, IRError> {
        let Node::FunctionDefinition { name, external, parameters, return_type, body } = node.data.clone() else {
            unreachable!()
        };

        let mut new_body = Vec::new();
        for child in body.iter() {
            new_body.push(self.generate_function_definition_body(child.clone())?);
        }

        Ok(node.convert(Node::FunctionDefinition { 
            name: name.clone(), 
            external: external.clone(), 
            parameters: parameters.clone(), 
            return_type: return_type.clone(), 
            body: new_body 
        }))
    }

    fn generate_function_definition_body(&mut self, node: Positioned<Node>) -> Result<Positioned<Node>, IRError> {
        match node.data {
            Node::Value(_) => self.generate_value(node),
            Node::FunctionDefinition { .. } => Err(IRError::UnexpectedNode(node, None)),
            Node::FunctionCall { .. } => self.generate_function_call(node),
            Node::Use(_) => Err(IRError::UnexpectedNode(node, None)),
        }
    }

    pub fn generate(&mut self) -> Result<IROutput, IRError> {
        let mut output = IROutput {
            includes: Vec::new(),
            ast: Vec::new(),
        };

        while let Some(current) = self.current() {
            match current.data {
                Node::Value(_) => return Err(IRError::UnexpectedNode(current, None)),
                Node::FunctionDefinition { .. } => output.ast.push(self.generate_function_definition(current)?),
                Node::FunctionCall { .. } => return Err(IRError::UnexpectedNode(current, None)),
                Node::Use(path) => {
                    for include in output.includes.iter() {
                        if include.full_path() == format!("{}.h", path.data) {
                            return Err(IRError::FileAlreadyIncluded(path, include.path.convert(())));
                        }
                    }

                    output.includes.push(self.generate_include(path)?);
                }
            }
            self.advance();
        } 
    
        Ok(output)
    }

}