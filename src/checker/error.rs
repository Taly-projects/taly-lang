////////////////////////////////////////////////////////////////////////////////////////////////////
//                                          Checker Error                                         //
//////////////////////////////////////////////////////////////////////////////////////////////////// 

use crate::util::{position::Positioned, source_file::SourceFile, error::{ErrorFormat, ErrorType}};

pub enum CheckerError {
    SymbolNotFound(Positioned<String>),
    UnexpectedType(Positioned<Option<String>>, Option<Positioned<String>>),
    TooManyParameters(usize, usize, Positioned<String>, Positioned<()>),
    NotEnoughParameters(usize, usize, Positioned<String>, Positioned<()>),
    VariableNotInitialized(Positioned<String>),
    CannotAssignToConstantExpression(Positioned<()>),
    CannotAssignToConstant(Positioned<()>, Positioned<String>),
    CannotInferType(Positioned<String>),
    CannotAccessAnythingHere(Positioned<()>),
    CannotAccessPrivateMember(Positioned<()>, Positioned<()>),
    CannotAccessProtectedMember(Positioned<()>, Positioned<()>),
    BreakStatementShouldOnlyBeFoundInLoops(Positioned<()>),
    ContinueStatementShouldOnlyBeFoundInLoops(Positioned<()>),
    LabelNotFound(Positioned<String>),
    FunctionNotImplemented(Positioned<String>, Positioned<String>),
    FunctionNotMatching(Positioned<String>, Positioned<String>, Positioned<()>),
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
            CheckerError::VariableNotInitialized(name) => {
                ErrorFormat::new(ErrorType::Error)
                    .add_message(format!("Variable '{}' not initialized: ", name.data), Some(name.convert(())))
                    .set_step("Checker".to_string()).print(src);
            }
            CheckerError::CannotAssignToConstantExpression(expr) => {
                ErrorFormat::new(ErrorType::Error)
                    .add_message(format!("Cannot assign to constant expression! "), Some(expr.clone()))
                    .set_step("Checker".to_string()).print(src);
            }
            CheckerError::CannotAssignToConstant(expr, constant) => {
                ErrorFormat::new(ErrorType::Error)
                    .add_message(format!("Cannot assign to constant '{}'! ", constant.data), Some(expr.clone()))
                    .add_message(format!("Defined here: "), Some(constant.convert(())))
                    .set_step("Checker".to_string()).print(src);
            }
            CheckerError::CannotInferType(var) => {
                ErrorFormat::new(ErrorType::Error)
                    .add_message(format!("Cannot infer type of '{}'! ", var.data), Some(var.convert(())))
                    .set_step("Checker".to_string()).print(src);
            }
            CheckerError::CannotAccessAnythingHere(node) => {
                ErrorFormat::new(ErrorType::Error)
                    .add_message(format!("Cannot selected anything here:"), Some(node.clone()))
                    .set_step("Checker".to_string()).print(src)
            }
            CheckerError::CannotAccessPrivateMember(node, definition) => {
                ErrorFormat::new(ErrorType::Error)
                    .add_message(format!("Cannot access private member:"), Some(node.clone()))
                    .add_message(format!("Defined private here:"), Some(definition.clone()))
                    .set_step("Checker".to_string()).print(src)
            },
            CheckerError::CannotAccessProtectedMember(node, definition) => {
                ErrorFormat::new(ErrorType::Error)
                    .add_message(format!("Cannot access protected member:"), Some(node.clone()))
                    .add_message(format!("Defined private here:"), Some(definition.clone()))
                    .set_step("Checker".to_string()).print(src)
            },
            CheckerError::BreakStatementShouldOnlyBeFoundInLoops(node) => {
                ErrorFormat::new(ErrorType::Error)
                    .add_message(format!("Break statement should only be found in loops!"), Some(node.clone()))
                    .set_step("Checker".to_string()).print(src)
            },
            CheckerError::ContinueStatementShouldOnlyBeFoundInLoops(node) => {
                ErrorFormat::new(ErrorType::Error)
                    .add_message(format!("Continue statement should only be found in loops!"), Some(node.clone()))
                    .set_step("Checker".to_string()).print(src)
            }
            CheckerError::LabelNotFound(name) => {
                ErrorFormat::new(ErrorType::Error)
                    .add_message(format!("Label '{}' not found!", name.data), Some(name.convert(())))
                    .set_step("Checker".to_string()).print(src)
            }
            CheckerError::FunctionNotImplemented(fun, from) => {
                ErrorFormat::new(ErrorType::Error)
                    .add_message(format!("Function '{}' is not implemented!", fun.data), Some(fun.convert(())))
                    .add_message(format!("From '{}'", from.data), Some(from.convert(())))
                    .set_step("Checker".to_string()).print(src)
            }
            CheckerError::FunctionNotMatching(fun, from, defined) => {
                ErrorFormat::new(ErrorType::Error)
                    .add_message(format!("Function '{}' is not implemented correctly!", fun.data), Some(fun.convert(())))
                    .add_message(format!("From '{}':", from.data), Some(from.convert(())))
                    .add_message(format!("Defined here:"), Some(defined.convert(())))
                    .set_step("Checker".to_string()).print(src)
            }
        }
    }

}