////////////////////////////////////////////////////////////////////////////////////////////////////
//                                              Token                                             //
//////////////////////////////////////////////////////////////////////////////////////////////////// 

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Token {
    Keyword(Keyword),
    Identifier(String),
    String(String),
    // Symbols
    LeftParenthesis,
    RightParenthesis,
    Comma,
    Colon,
    RightDoubleArrow,
    // Formatting
    Tab,
    NewLine
}



////////////////////////////////////////////////////////////////////////////////////////////////////
//                                             Keyword                                            //
//////////////////////////////////////////////////////////////////////////////////////////////////// 

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Keyword {
    Use,
    Fn,
    Extern
}

impl Keyword {

    pub fn from_string(str: &str) -> Option<Keyword> {
        match str {
            "use" => Some(Keyword::Use),
            "fn" => Some(Keyword::Fn),
            "extern" => Some(Keyword::Extern),
            _ => None
        }
    }

}