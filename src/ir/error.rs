use crate::{util::{position::Positioned, error::{ErrorFormat, ErrorType}, source_file::SourceFile}, parser::node::Node};

////////////////////////////////////////////////////////////////////////////////////////////////////
//                                           Lexer Error                                          //
//////////////////////////////////////////////////////////////////////////////////////////////////// 

pub enum IRError {
    UnexpectedNode(Positioned<Node>, Option<String>),
    FileAlreadyIncluded(Positioned<String>, Positioned<()>)
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
                ErrorFormat::new(ErrorType::Error).add_message(buf, Some(found.convert(()))).set_step("IR Generator".to_string()).print(src);
            },
            IRError::FileAlreadyIncluded(found, previous) => {
                ErrorFormat::new(ErrorType::Error)
                    .add_message(format!("File '{}' already included!", found.data), Some(found.convert(())))
                    .add_message("previously included here:".to_string(), Some(previous.clone()))
                    .set_step("IR Generator".to_string()).print(src);  
            }
        }
    }

}