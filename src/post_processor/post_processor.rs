use crate::{ir::output::IROutput, util::position::Positioned, parser::node::{Node, Operator}};

pub struct PostProcessor {
    ir_output: IROutput,
    index: usize
}

impl PostProcessor {

    pub fn new(ir_output: IROutput) -> Self {
        Self {
            ir_output,
            index: 0
        }
    } 

    fn current(&self) -> Option<Positioned<Node>> {
        self.ir_output.ast.get(self.index).cloned()
    }

    fn advance(&mut self) {
        self.index += 1;
    }

    fn process_function_definition(&mut self, node: Positioned<Node>) -> Positioned<Node> {
        let Node::FunctionDefinition { name, external, constructor, parameters, return_type, body, access  } = node.data.clone() else {
            unreachable!()
        };

        let mut new_body = Vec::new();
        for node in body {
            new_body.push(self.process_node(node));
        }

        node.convert(Node::FunctionDefinition { 
            name, 
            external, 
            constructor, 
            parameters, 
            return_type, 
            body: new_body, 
            access 
        })
    }

    fn process_function_call(&mut self, node: Positioned<Node>) -> Positioned<Node> {
        let Node::FunctionCall { name, parameters } = node.data.clone() else {
            unreachable!()
        };

        let mut new_params = Vec::new();
        for param in parameters {
            new_params.push(self.process_node(param));
        }

        node.convert(Node::FunctionCall { 
            name, 
            parameters: new_params
        })
    }

    fn process_variable_definition(&mut self, node: Positioned<Node>) -> Positioned<Node> {
        let Node::VariableDefinition { var_type, name, data_type, value, access } = node.data.clone() else {
            unreachable!()
        };

        node.convert(Node::VariableDefinition { 
            var_type, 
            name, 
            data_type, 
            value: value.map(|x| Box::new(self.process_node(*x))), 
            access 
        })
    }

    fn process_access(&mut self, node: Positioned<Node>) -> Positioned<Node> {
        let Node::BinaryOperation { lhs, operator, rhs } = node.data.clone() else {
            unreachable!()
        };

        if let Node::FunctionCall { .. } = rhs.data.clone() {
            self.process_node(*rhs)            
        } else {
            node.convert(Node::BinaryOperation { 
                lhs: Box::new(self.process_node(*lhs)), 
                operator: operator, 
                rhs: Box::new(self.process_node(*rhs)) 
            })
        }
    }

    fn process_bin_op(&mut self, node: Positioned<Node>) -> Positioned<Node> {
        let Node::BinaryOperation { lhs, operator, rhs } = node.data.clone() else {
            unreachable!()
        };

        node.convert(Node::BinaryOperation { 
            lhs: Box::new(self.process_node(*lhs)), 
            operator, 
            rhs: Box::new(self.process_node(*rhs)) 
        })
    }

    fn process_return(&mut self, node: Positioned<Node>) -> Positioned<Node> {
        let Node::Return(expr) = node.data.clone() else {
            unreachable!()
        };

        node.convert(Node::Return(expr.map(|x| Box::new(self.process_node(*x)))))
    }

    fn process_class_definition(&mut self, node: Positioned<Node>) -> Positioned<Node> {
        let Node::ClassDefinition { name, body, access } = node.data.clone() else {
            unreachable!()
        };

        let mut new_body = Vec::new();
        for node in body {
            new_body.push(self.process_node(node));
        }

        node.convert(Node::ClassDefinition { 
            name, 
            body: new_body, 
            access 
        })
    }

    fn process_space_definition(&mut self, node: Positioned<Node>) -> Positioned<Node> {
        let Node::SpaceDefinition { name, body, access } = node.data.clone() else {
            unreachable!()
        };

        let mut new_body = Vec::new();
        for node in body {
            new_body.push(self.process_node(node));
        }

        node.convert(Node::SpaceDefinition { 
            name, 
            body: new_body, 
            access 
        })
    }

    fn process_node(&mut self, node: Positioned<Node>) -> Positioned<Node> {
        match node.data.clone() {
            Node::Value(_) => node,
            Node::FunctionDefinition { .. } => self.process_function_definition(node),
            Node::FunctionCall { .. } => self.process_function_call(node),
            Node::Use(_) => node,
            Node::VariableDefinition { .. } => self.process_variable_definition(node),
            Node::VariableCall(_) => node,
            Node::BinaryOperation { operator, .. } if operator.data == Operator::Access => self.process_access(node),
            Node::BinaryOperation { .. } => self.process_bin_op(node),
            Node::Return(_) => self.process_return(node),
            Node::ClassDefinition { .. } => self.process_class_definition(node),
            Node::SpaceDefinition { .. } => self.process_space_definition(node),
            Node::_Unchecked(inner) => self.process_node(*inner),
            Node::_Optional(inner) => self.process_node(*inner),
        }
    }

    pub fn process(&mut self) -> IROutput {
        let mut output = IROutput {
            includes: self.ir_output.includes.clone(),
            ast: Vec::new(),
        };

        while let Some(current) = self.current() {
            output.ast.push(self.process_node(current));
            self.advance();
        }

        output
    }

}