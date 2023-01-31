////////////////////////////////////////////////////////////////////////////////////////////////////
//                                              Token                                             //
//////////////////////////////////////////////////////////////////////////////////////////////////// 

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Token {
    Keyword(Keyword),
    Identifier(String),
    String(String),
    Integer(String),
    Decimal(String),
    Bool(bool),
    // Symbols
    LeftParenthesis,
    RightParenthesis,
    Comma,
    Colon,
    RightDoubleArrow,
    Equal,
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
    Extern,
    Var,
    Const
}

impl Keyword {

    pub fn from_string(str: &str) -> Option<Keyword> {
        match str {
            "use" => Some(Keyword::Use),
            "fn" => Some(Keyword::Fn),
            "extern" => Some(Keyword::Extern),
            "var" => Some(Keyword::Var),
            "const" => Some(Keyword::Const),
            _ => None
        }
    }

}