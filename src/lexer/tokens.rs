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
    Plus,
    Dash,
    Star,
    Slash,
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
    Const,
    Return,
    Class,
    Space
}

impl Keyword {

    pub fn from_string(str: &str) -> Option<Keyword> {
        match str {
            "use" => Some(Keyword::Use),
            "fn" => Some(Keyword::Fn),
            "extern" => Some(Keyword::Extern),
            "var" => Some(Keyword::Var),
            "const" => Some(Keyword::Const),
            "return" => Some(Keyword::Return),
            "class" => Some(Keyword::Class),
            "space" => Some(Keyword::Space),
            _ => None
        }
    }

}