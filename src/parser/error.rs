use crate::{util::{position::Positioned, source_file::SourceFile, error::{ErrorFormat, ErrorType}}, lexer::tokens::Token};

////////////////////////////////////////////////////////////////////////////////////////////////////
//                                           Parser Error                                         //
//////////////////////////////////////////////////////////////////////////////////////////////////// 

pub enum ParserError {
    UnexpectedToken(Positioned<Token>, Option<String>),
    UnexpectedEOF(Option<String>)
}

impl ParserError {

    pub fn print_error(&self, src: &SourceFile) {
        match self {
            ParserError::UnexpectedToken(found, expected) => {
                let mut buf = format!("Unexpected token '{:?}'", found.data);
                if let Some(expected) = expected {
                    buf.push_str(format!(", should be '{}'!", expected).as_str());
                } else {
                    buf.push('!');
                }
                ErrorFormat::new(ErrorType::Error).set_message(buf).set_step("Parser".to_string()).set_pos(found.convert(())).print(src);
            },
            ParserError::UnexpectedEOF(expected) => {
                let mut buf = "Unexpected EOF".to_string();
                if let Some(expected) = expected {
                    buf.push_str(format!(", should be '{}'!", expected).as_str());
                } else {
                    buf.push('!');
                }
                ErrorFormat::new(ErrorType::Error).set_message(buf).set_step("parser".to_string()).print(src);
            },
        }
    }

}