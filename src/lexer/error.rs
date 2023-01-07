use crate::util::{position::Positioned, error::{ErrorFormat, ErrorType}, source_file::SourceFile};

////////////////////////////////////////////////////////////////////////////////////////////////////
//                                           Lexer Error                                          //
//////////////////////////////////////////////////////////////////////////////////////////////////// 

pub enum LexerError {
    UnexpectedChar(Positioned<char>, Option<String>),
    UnexpectedEOF(Option<String>)
}

impl LexerError {

    pub fn print_error(&self, src: &SourceFile) {
        match self {
            LexerError::UnexpectedChar(found, expected) => {
                let mut buf = format!("Unexpected char '{}'", found.data);
                if let Some(expected) = expected {
                    buf.push_str(format!(", should be '{}'!", expected).as_str());
                } else {
                    buf.push('!');
                }
                ErrorFormat::new(ErrorType::Error).set_message(buf).set_step("Lexer".to_string()).set_pos(found.convert(())).print(src);
            },
            LexerError::UnexpectedEOF(expected) => {
                let mut buf = "Unexpected EOF".to_string();
                if let Some(expected) = expected {
                    buf.push_str(format!(", should be '{}'!", expected).as_str());
                } else {
                    buf.push('!');
                }
                ErrorFormat::new(ErrorType::Error).set_message(buf).set_step("Lexer".to_string()).print(src);
            },
        }
    }

}