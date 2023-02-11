////////////////////////////////////////////////////////////////////////////////////////////////////
//                                        Symbolizer Error                                        //
//////////////////////////////////////////////////////////////////////////////////////////////////// 

use crate::util::{position::Positioned, source_file::SourceFile, error::{ErrorFormat, ErrorType}};

pub enum SymbolizerError {
    SymbolAlreadyDefined(Positioned<String>, Positioned<()>),
    SymbolNotFound(Positioned<String>)
}

impl SymbolizerError {

    pub fn print_error(&self, src: &SourceFile) {
        match self {
            SymbolizerError::SymbolAlreadyDefined(symbol, here) => {
                ErrorFormat::new(ErrorType::Error)
                    .add_message(format!("Symbol '{}' already defined:", symbol.data), Some(symbol.convert(())))
                    .add_message(format!("Defined here:"), Some(here.convert(())))
                    .set_step("Symbolizer".to_string()).print(src);
            },
            SymbolizerError::SymbolNotFound(symbol) => {
                ErrorFormat::new(ErrorType::Error)
                    .add_message(format!("Symbol '{}' not found:", symbol.data), Some(symbol.convert(())))
                    .set_step("Symbolizer".to_string()).print(src);
            },
        }
    }

}