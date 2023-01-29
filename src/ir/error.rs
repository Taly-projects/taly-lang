use crate::{util::{position::Positioned, error::{ErrorFormat, ErrorType}, source_file::SourceFile}, parser::node::Node};

////////////////////////////////////////////////////////////////////////////////////////////////////
//                                           Lexer Error                                          //
//////////////////////////////////////////////////////////////////////////////////////////////////// 

pub enum IRError {
    UnexpectedNode(Positioned<Node>, Option<String>)
}

impl IRError {

    pub fn print_error(&self, src: &SourceFile) {
        match self {
            IRError::UnexpectedNode(found, expected) => {
                let mut buf = format!("Unexpected node '{}'", found.data.short_name());
                if let Some(expected) = expected {
                    buf.push_str(format!(", should be '{}'!", expected).as_str());
                } else {
                    buf.push('!'); 
                }
                ErrorFormat::new(ErrorType::Error).set_message(buf).set_step("IR Generator".to_string()).set_pos(found.convert(())).print(src);
            },
        }
    }

}