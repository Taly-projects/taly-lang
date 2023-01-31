////////////////////////////////////////////////////////////////////////////////////////////////////
//                                          Checker Error                                         //
//////////////////////////////////////////////////////////////////////////////////////////////////// 

use crate::util::{position::Positioned, source_file::SourceFile, error::{ErrorFormat, ErrorType}};

pub enum CheckerError {
    SymbolNotFound(Positioned<String>),
    UnexpectedType(Positioned<Option<String>>, Option<Positioned<String>>),
    TooManyParameters(usize, usize, Positioned<String>, Positioned<()>),
    NotEnoughParameters(usize, usize, Positioned<String>, Positioned<()>),
}

impl CheckerError {

    pub fn print_error(&self, src: &SourceFile) {
        match self {
            CheckerError::SymbolNotFound(symbol) => {
                ErrorFormat::new(ErrorType::Error)
                    .add_message(format!("Symbol '{}' not found", symbol.data), Some(symbol.convert(())))
                    .set_step("Checker".to_string()).print(src);
            },
            CheckerError::UnexpectedType(found, expected) => {
                let mut error_msg = ErrorFormat::new(ErrorType::Error)
                    .add_message(format!("Unexpected type '{}':", found.data.clone().unwrap_or("-NoType".to_string())), Some(found.convert(())))
                    .set_step("Checker".to_string());

                if let Some(expected) = expected {
                    error_msg = error_msg.add_message(format!("Should be '{}':", expected.data), Some(expected.convert(())))
                }

                error_msg.print(&src);
            }
            CheckerError::TooManyParameters(found, expected, call, definition) => {
                ErrorFormat::new(ErrorType::Error)
                    .add_message(format!("Too many parameters, found '{}', expected '{}', for '{}'", found, expected, call.data), Some(call.convert(())))
                    .add_message(format!("Definition here:"), Some(definition.clone()))
                    .set_step("Checker".to_string()).print(src);
            },
            CheckerError::NotEnoughParameters(found, expected, call, definition) => {
                ErrorFormat::new(ErrorType::Error)
                    .add_message(format!("Not enough parameters, found '{}', expected '{}', for '{}'", found, expected, call.data), Some(call.convert(())))
                    .add_message(format!("Definition here:"), Some(definition.clone()))
                    .set_step("Checker".to_string()).print(src);
            },
        }
    }

}