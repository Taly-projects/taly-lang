use crate::{util::{position::Positioned, error::{ErrorFormat, ErrorType}, source_file::SourceFile}, parser::node::Node};

////////////////////////////////////////////////////////////////////////////////////////////////////
//                                            IR Error                                            //
//////////////////////////////////////////////////////////////////////////////////////////////////// 

pub enum IRError {
    UnexpectedNode(Positioned<Node>, Option<String>),
    FileAlreadyIncluded(Positioned<String>, Positioned<()>),
    CannotSpecifyAccessHere(Positioned<()>),
    DestructorAlreadyDefined(Positioned<()>, Positioned<()>),
    DestructorShouldNotReturnAnything(Positioned<()>),
    DestructorShouldNotHaveParameters(Positioned<()>),
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
            IRError::CannotSpecifyAccessHere(node) => {
                ErrorFormat::new(ErrorType::Error)
                    .add_message(format!("Cannot specify access here:"), Some(node.clone()))
                    .set_step("IR Generator".to_string()).print(src);  
            },
            IRError::DestructorAlreadyDefined(found, previous) => {
                ErrorFormat::new(ErrorType::Error)
                    .add_message(format!("Destructor already defined:"), Some(found.clone()))
                    .add_message("previously defined here:".to_string(), Some(previous.clone()))
                    .set_step("IR Generator".to_string()).print(src);  
            },
            IRError::DestructorShouldNotReturnAnything(node) => {
                ErrorFormat::new(ErrorType::Error)
                    .add_message(format!("Destructor should not return anything"), Some(node.clone()))
                    .set_step("IR Generator".to_string()).print(src);  
            },
            IRError::DestructorShouldNotHaveParameters(node) => {
                ErrorFormat::new(ErrorType::Error)
                    .add_message(format!("Destructor should not have parameters"), Some(node.clone()))
                    .set_step("IR Generator".to_string()).print(src);  
            },
        }
    }

}