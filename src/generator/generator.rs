use crate::{ir::output::{IROutput, IncludeType}, generator::project::Project, util::position::Positioned, parser::node::{Node, ValueNode}};

pub struct Generator {
    ir_output: IROutput,
    index: usize
}

impl Generator {

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

    fn generate_value(&mut self, node: Positioned<Node>) -> (bool, String) {
        let Node::Value(value) = node.data.clone() else {
            unreachable!()
        };

        match value {
            ValueNode::String(str) => (true, format!("\"{}\"", str)),
        }
    }

    fn generate_function_call(&mut self, node: Positioned<Node>) -> (bool, String) {
        let Node::FunctionCall { name, parameters } = node.data.clone() else {
            unreachable!()
        };

        let mut buf = String::new();
        buf.push_str(&name.data);
        buf.push('(');
        let mut index = 0;
        for param in parameters {
            if index != 0 {
                buf.push_str(", ");
            }
            buf.push_str(&self.generate_current(param).1);
            index += 1;
        }
        buf.push(')');

        (true, buf)
    }

    fn generate_current(&mut self, node: Positioned<Node>) -> (bool, String) {
        match node.data {
            Node::Value(_) => self.generate_value(node),
            Node::FunctionCall { .. } => self.generate_function_call(node),
            _ => unreachable!(),
        }
    }

    fn generate_root_function_definition(&mut self, node: Positioned<Node>, project: &mut Project) {
        let Node::FunctionDefinition { name, external, parameters, return_type, body } = node.data.clone() else {
            unreachable!()
        };

        if external {
            return;
        }

        let mut function_header = String::new();
        function_header.push_str(&return_type.map_or("void".to_string(), |x| x.data));
        function_header.push(' ');
        function_header.push_str(&name.data);
        function_header.push('(');
        let mut index = 0;
        for param in parameters {
            if index != 0 {
                function_header.push_str(", ");
            }
            function_header.push_str(&param.data_type.data);
            function_header.push(' ');
            function_header.push_str(&param.name.data);
            index += 1;
        }
        function_header.push(')');

        let file = project.get_file("main".to_string());
        if name.data != "main" {
            file.header.push_str(&function_header);
            file.header.push_str(";\n");
        }

        file.src.push_str(&function_header);
        file.src.push_str(" { ");
        for node in body.clone() {
            let node_str = self.generate_current(node);
            for line in node_str.1.lines() {
                file.src.push_str("\n\t");
                file.src.push_str(line);
            }
            if node_str.0 {
                file.src.push(';');
            }
        }
        if !body.is_empty() {
            file.src.push('\n');
        }
        file.src.push('}');
        file.src.push('\n');
    }

    pub fn generate(&mut self) -> Project {
        let mut project = Project::new();

        while let Some(node) = self.current() {
            match node.data {
                Node::FunctionDefinition { .. } => self.generate_root_function_definition(node, &mut project),
                _ => unreachable!()
            }
            self.advance();
        }

        // Process Includes
        let mut include_buf = String::new();
        for include in self.ir_output.includes.iter() {
            include_buf.push_str("#include "); 
            match include.include_type {
                IncludeType::External => include_buf.push_str(&format!("\"{}\"", include.path.data)),
                IncludeType::StdExternal => include_buf.push_str(&format!("<{}>", include.path.data)),
                IncludeType::Internal => include_buf.push_str(&format!("\"{}\"", include.path.data)),
            }      
            include_buf.push('\n');
        }

        for file in project.files.iter_mut() {
            file.header = format!("{}\n{}", include_buf, file.header);
            if !file.src.is_empty() {
                file.src = format!("#include \"{}.h\"\n\n{}", file.name, file.src);
            }
        }

        project
    }

}