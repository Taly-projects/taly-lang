use crate::{ir::output::{IROutput, IncludeType}, generator::project::{Project, File}, util::position::Positioned, parser::node::{Node, ValueNode, Operator}};

////////////////////////////////////////////////////////////////////////////////////////////////////
//                                           Generator                                         //
//////////////////////////////////////////////////////////////////////////////////////////////////// 

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

    fn generate_type(&mut self, data_type: String) -> String {
        match data_type.as_str() {
            "c_string" => "const char*".to_string(),
            "c_int" => "int".to_string(),
            "c_float" => "float".to_string(),
            "void" => "void".to_string(),
            _ => {
                if data_type.starts_with("_NOPTR_") {
                    (&data_type[7..data_type.len()]).to_string()
                } else {
                    format!("{}*", data_type)
                }
            }
        }
    }

    fn generate_value(&mut self, node: Positioned<Node>) -> (bool, String) {
        let Node::Value(value) = node.data.clone() else {
            unreachable!()
        };

        match value {
            ValueNode::String(str) => (true, format!("\"{}\"", str)),
            ValueNode::Bool(b) => (true, format!("{}", b)),
            ValueNode::Integer(num) => (true, format!("{}", num)),
            ValueNode::Decimal(num) => (true, format!("{}", num)),
            ValueNode::Type(str) => (true, self.generate_type(str)),
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

    fn generate_variable_definition(&mut self, node: Positioned<Node>) -> (bool, String) {
        let Node::VariableDefinition { name, data_type, value, .. } = node.data.clone() else {
            unreachable!()
        };

        let mut buf = String::new();

        buf.push_str(&self.generate_type(data_type.expect("No type could be inferred!").data));
        buf.push(' ');
        buf.push_str(&name.data);
        
        if let Some(value) = value {
            buf.push_str(" = ");
            buf.push_str(&self.generate_current(*value).1);
        }

        (true, buf)
    }

    fn generate_binary_operation(&mut self, node: Positioned<Node>) -> (bool, String) {
        let Node::BinaryOperation { lhs, operator, rhs } = node.data.clone() else {
            unreachable!()
        };

        let mut buf = String::new();

        buf.push('(');
        buf.push_str(&self.generate_current(*lhs).1);
        match operator.data {
            Operator::Add => buf.push_str(" + "),
            Operator::Subtract => buf.push_str(" - "),
            Operator::Multiply => buf.push_str(" * "),
            Operator::Divide => buf.push_str(" / "),
            Operator::Assign => buf.push_str(" = "),
            Operator::Access => buf.push_str("->"),
        }
        buf.push_str(&self.generate_current(*rhs).1);
        buf.push(')');

        (true, buf)
    }

    fn generate_variable_call(&mut self, node: Positioned<Node>) -> (bool, String) {
        let Node::VariableCall(name) = node.data.clone() else {
            unreachable!()
        };

        let buf = format!("{}", name.clone());

        (true, buf)
    }

    fn generate_return(&mut self, node: Positioned<Node>) -> (bool, String) {
        let Node::Return(expr) = node.data.clone() else {
            unreachable!()
        };

        let mut buf = String::new();
        buf.push_str("return");
        if let Some(expr) = expr {
            buf.push(' ');
            buf.push_str(&self.generate_current(*expr).1);
        }

        (true, buf)
    }

    fn generate_current(&mut self, node: Positioned<Node>) -> (bool, String) {
        match node.data {
            Node::Value(_) => self.generate_value(node),
            Node::FunctionCall { .. } => self.generate_function_call(node),
            Node::VariableDefinition { .. } => self.generate_variable_definition(node),
            Node::VariableCall(_) => self.generate_variable_call(node),
            Node::BinaryOperation { .. } => self.generate_binary_operation(node),
            Node::Return(_) => self.generate_return(node),
            _ => unreachable!(),
        }
    }

    fn generate_root_function_definition(&mut self, node: Positioned<Node>) -> File {
        let Node::FunctionDefinition { name, external, parameters, return_type, body, .. } = node.data.clone() else {
            unreachable!()
        };

        if external {
            return File::new("_".to_string());
        }

        let mut function_header = String::new();
        function_header.push_str(&self.generate_type(return_type.map_or("void".to_string(), |x| x.data)));
        function_header.push(' ');
        function_header.push_str(&name.data);
        function_header.push('(');
        let mut index = 0;
        for param in parameters {
            if index != 0 {
                function_header.push_str(", ");
            }
            function_header.push_str(&self.generate_type(param.data_type.data));
            function_header.push(' ');
            function_header.push_str(&param.name.data);
            index += 1;
        }
        function_header.push(')');

        let mut file = File::new("_".to_string());
        if name.data != "main" {
            file.header.push_str(&function_header);
            file.header.push_str(";\n\n");
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
        file.src.push('\n');

        file
    }

    fn generate_class_definition(&mut self, node: Positioned<Node>, file: &mut File) {
        let Node::ClassDefinition { name, body, .. } = node.data.clone() else {
            unreachable!()
        };

        // Separate fields and methods
        let mut fields = Vec::new();
        let mut methods = Vec::new();
        for node in body.iter() {
            match node.data {
                Node::FunctionDefinition { .. } => methods.push(node.clone()),
                Node::VariableDefinition { .. } => fields.push(node.clone()),
                _ => unreachable!()
            }
        }

        // Create Structure
        let mut struct_buf = String::new();
        struct_buf.push_str("typedef struct ");
        struct_buf.push_str(&name.data);
        struct_buf.push_str(" { ");
        if !fields.is_empty() {
            struct_buf.push_str("\n");
        }
        for field in fields.iter() {
            let Node::VariableDefinition { name, data_type, .. } = field.data.clone() else {
                unreachable!()
            };

            struct_buf.push_str("\t");
            struct_buf.push_str(&self.generate_type(data_type.expect("No Type Could be inferred!").data));
            struct_buf.push_str(" ");
            struct_buf.push_str(&name.data);
            struct_buf.push_str(";");
            struct_buf.push_str("\n");
        }
        struct_buf.push_str("} ");
        struct_buf.push_str(&name.data);
        struct_buf.push_str(";\n\n");

        // let file = project.get_file(name.data.clone());
        file.header.push_str(&struct_buf);

        for method in methods.iter() {
            let fun_file = self.generate_root_function_definition(method.clone());
            file.header.push_str(&fun_file.header);
            file.src.push_str(&fun_file.src);
        }
    }


    fn generate_space_definition(&mut self, node: Positioned<Node>, file: &mut File) {
        let Node::SpaceDefinition { body, .. } = node.data.clone() else {
            unreachable!()
        };

        // Separate methods
        let mut methods = Vec::new();
        let mut spaces = Vec::new();
        let mut classes = Vec::new();
        for node in body.iter() {
            match node.data {
                Node::FunctionDefinition { .. } => methods.push(node.clone()),
                Node::SpaceDefinition { .. } => spaces.push(node.clone()),
                Node::ClassDefinition { .. } => classes.push(node.clone()),
                _ => unreachable!()
            }
        }

        // let file = project.get_file(name.data.clone());

        for method in methods.iter() {
            let fun_file = self.generate_root_function_definition(method.clone());
            file.header.push_str(&fun_file.header);
            file.src.push_str(&fun_file.src);
        }

        for class in classes.iter() {
            self.generate_class_definition(class.clone(), file);
        }

        for space in spaces.iter() {
            self.generate_space_definition(space.clone(), file);
        }
    }

    pub fn generate(&mut self) -> Project {
        let mut project = Project::new();

        while let Some(node) = self.current() {
            match node.data {
                Node::FunctionDefinition { .. } => {
                    let file = self.generate_root_function_definition(node);
                    let main_file = project.get_file("main".to_string());
                    main_file.header.push_str(&file.header);
                    main_file.src.push_str(&file.src);
                }
                Node::ClassDefinition { .. } => self.generate_class_definition(node, project.get_file("main".to_string())),
                Node::SpaceDefinition { .. } => self.generate_space_definition(node, project.get_file("main".to_string())),
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