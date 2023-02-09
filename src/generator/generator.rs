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

    pub fn generate_type(data_type: String) -> String {
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
            ValueNode::Type(str) => (true, Self::generate_type(str)),
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

        buf.push_str(&Self::generate_type(data_type.expect("No type could be inferred!").data));
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
            Operator::BooleanAnd => buf.push_str(" && "),
            Operator::BooleanOr => buf.push_str(" || "),
            Operator::Equal => buf.push_str(" == "),
            Operator::NotEqual => buf.push_str(" != "),
            Operator::Less => buf.push_str(" < "),
            Operator::LessOrEqual => buf.push_str(" <= "),
            Operator::Greater => buf.push_str(" > "),
            Operator::GreaterOrEqual => buf.push_str(" >= "),
            _ => unreachable!()
        }
        buf.push_str(&self.generate_current(*rhs).1);
        buf.push(')');

        (true, buf)
    }

    fn generate_unary_operation(&mut self, node: Positioned<Node>) -> (bool, String) {
        let Node::UnaryOperation { operator, value } = node.data.clone() else {
            unreachable!()
        };

        let mut buf = String::new();

        buf.push('(');
        match operator.data {
            Operator::Add => buf.push_str("+"),
            Operator::Subtract => buf.push_str("-"),
            Operator::BooleanNot => buf.push_str("!"),
            _ => unreachable!()
        }
        buf.push_str(&self.generate_current(*value).1);
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

    fn generate_break(&mut self, node: Positioned<Node>) -> (bool, String) {
        let Node::Break(label) = node.data.clone() else {
            unreachable!()
        };

        let mut buf = String::new();
        buf.push_str("break");
        if let Some(label) = label {
            buf.push(' ');
            buf.push_str(&label.data);
        }

        (true, buf)
    }

    fn generate_continue(&mut self, node: Positioned<Node>) -> (bool, String) {
        let Node::Continue(label) = node.data.clone() else {
            unreachable!()
        };

        let mut buf = String::new();
        buf.push_str("continue");
        if let Some(label) = label {
            buf.push(' ');
            buf.push_str(&label.data);
        }
        
        (true, buf)
    }

    fn generate_label(&mut self, node: Positioned<Node>) -> (bool, String) {
        let Node::Label { name, inner } = node.data.clone() else {
            unreachable!()
        };

        let mut buf = String::new();
        buf.push_str(&name.data);
        buf.push_str(": ");
        let inner_out = &self.generate_current(*inner);
        buf.push_str(&inner_out.1);
        
        (inner_out.0, buf)
    }

    fn generate_if_statement(&mut self, node: Positioned<Node>) -> (bool, String) {
        let Node::IfStatement { condition, body, elif_branches, else_body } = node.data.clone() else {
            unreachable!()
        };

        let mut buf = String::new();
        buf.push_str("if (");
        buf.push_str(&self.generate_current(*condition).1);
        buf.push_str(") { ");
        for node in body.clone() {
            let node_str = self.generate_current(node);
            for line in node_str.1.lines() {
                buf.push_str("\n\t");
                buf.push_str(line);
            }
            if node_str.0 {
                buf.push(';');
            }
        }
        if !body.is_empty() {
            buf.push('\n');
        }
        buf.push_str("} ");

        for elif_branch in elif_branches {
            buf.push_str("else if (");
            buf.push_str(&self.generate_current(elif_branch.condition).1);
            buf.push_str(") { ");
            for node in elif_branch.body.clone() {
                let node_str = self.generate_current(node);
                for line in node_str.1.lines() {
                    buf.push_str("\n\t");
                    buf.push_str(line);
                }
                if node_str.0 {
                    buf.push(';');
                }
            }
            if !elif_branch.body.is_empty() {
                buf.push('\n');
            }
            buf.push_str("} ");
        }

        if !else_body.is_empty() {
            buf.push_str("else {");
            for node in else_body.clone() {
                let node_str = self.generate_current(node);
                for line in node_str.1.lines() {
                    buf.push_str("\n\t");
                    buf.push_str(line);
                }
                if node_str.0 {
                    buf.push(';');
                }
            }
            if !else_body.is_empty() {
                buf.push('\n');
            }
            buf.push_str("} ");

        }

        (false, buf)
    }

    fn generate_while_loop(&mut self, node: Positioned<Node>) -> (bool, String) {
        let Node::WhileLoop { condition, body } = node.data.clone() else {
            unreachable!()
        };

        let mut buf = String::new();
        buf.push_str("while (");
        buf.push_str(&self.generate_current(*condition).1);
        buf.push_str(") { ");
        for node in body.clone() {
            let node_str = self.generate_current(node);
            for line in node_str.1.lines() {
                buf.push_str("\n\t");
                buf.push_str(line);
            }
            if node_str.0 {
                buf.push(';');
            }
        }
        if !body.is_empty() {
            buf.push('\n');
        }
        buf.push_str("} ");

        (false, buf)
    }

    fn generate_current(&mut self, node: Positioned<Node>) -> (bool, String) {
        match node.data {
            Node::Value(_) => self.generate_value(node),
            Node::FunctionCall { .. } => self.generate_function_call(node),
            Node::VariableDefinition { .. } => self.generate_variable_definition(node),
            Node::VariableCall(_) => self.generate_variable_call(node),
            Node::BinaryOperation { .. } => self.generate_binary_operation(node),
            Node::UnaryOperation { .. } => self.generate_unary_operation(node),
            Node::Return(_) => self.generate_return(node),
            Node::IfStatement { .. } => self.generate_if_statement(node),
            Node::WhileLoop { .. } => self.generate_while_loop(node),
            Node::Break(_) => self.generate_break(node),
            Node::Continue(_) => self.generate_continue(node),
            Node::Label { .. } => self.generate_label(node),
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
        function_header.push_str(&Self::generate_type(return_type.map_or("void".to_string(), |x| x.data)));
        function_header.push(' ');
        function_header.push_str(&name.data);
        function_header.push('(');
        let mut index = 0;
        for param in parameters {
            if index != 0 {
                function_header.push_str(", ");
            }
            function_header.push_str(&Self::generate_type(param.data_type.data));
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
            struct_buf.push_str(&Self::generate_type(data_type.expect("No Type Could be inferred!").data));
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
        let mut interfaces = Vec::new();
        for node in body.iter() {
            match node.data {
                Node::FunctionDefinition { .. } => methods.push(node.clone()),
                Node::SpaceDefinition { .. } => spaces.push(node.clone()),
                Node::ClassDefinition { .. } => classes.push(node.clone()),
                Node::InterfaceDefinition { .. } => interfaces.push(node.clone()),
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

        for interface in interfaces.iter() {
            self.generate_interface_definition(interface.clone(), file);
        }

        for space in spaces.iter() {
            self.generate_space_definition(space.clone(), file);
        }
    }

    fn generate_interface_definition(&mut self, node: Positioned<Node>, file: &mut File) {
        let Node::InterfaceDefinition { name, body, .. } = node.data.clone() else {
            unreachable!()
        };

        // Separate methods
        let mut methods = Vec::new();
        for node in body.iter() {
            match &node.data {
                Node::FunctionDefinition { body, .. } => {
                    methods.push(node.clone());
                    if !body.is_empty() {
                        todo!("Default functions not implement for interface yet!")
                    }
                }
                _ => unreachable!()
            }
        }

        // Create Structure
        let mut struct_buf = String::new();
        struct_buf.push_str("typedef struct ");
        struct_buf.push_str(&name.data);
        struct_buf.push_str(" { ");
        if !methods.is_empty() {
            struct_buf.push_str("\n");
        }
        for method in methods.iter() {
            let Node::FunctionDefinition { name, parameters, return_type, .. } = method.data.clone() else {
                unreachable!()
            };

            struct_buf.push_str("\t");
            struct_buf.push_str(&Self::generate_type(return_type.map_or("void".to_string(), |x| x.data)));
            struct_buf.push_str(" (*");
            struct_buf.push_str(&name.data);
            struct_buf.push_str(")(");
            let mut first = false;
            for param in parameters {
                if first {
                    struct_buf.push_str(", ");
                }
                struct_buf.push_str(&Self::generate_type(param.data_type.data));
                first = false;
            }
            struct_buf.push_str(");\n");
        }
        struct_buf.push_str("} ");
        struct_buf.push_str(&name.data);
        struct_buf.push_str(";\n\n");

        file.header.push_str(&struct_buf);

        // for method in methods.iter() {
        //     let fun_file = self.generate_root_function_definition(method.clone());
        //     file.header.push_str(&fun_file.header);
        //     file.src.push_str(&fun_file.src);
        // }
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
                Node::InterfaceDefinition { .. } => self.generate_interface_definition(node, project.get_file("main".to_string())),
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
            file.header = format!("#ifndef TALY_GEN_C_{0}_H\n#define TALY_GEN_C_{0}_H\n\n{1}\n{2}#endif // TALY_GEN_C_{0}_H", file.name, include_buf, file.header);
            if !file.src.is_empty() {
                file.src = format!("#include \"{}.h\"\n\n{}", file.name, file.src);
            }
        }

        project
    }

}