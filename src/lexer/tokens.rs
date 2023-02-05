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
    Dot,
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
    Space,
    New,
    Pub,
    Prot,
    Lock,
    Guard,
    And,
    Or,
    Xor,
    Not
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
            "new" => Some(Keyword::New),
            "pub" => Some(Keyword::Pub),
            "prot" => Some(Keyword::Prot),
            "lock" => Some(Keyword::Lock),
            "guard" => Some(Keyword::Guard),
            "and" => Some(Keyword::And),
            "or" => Some(Keyword::Or),
            "xor" => Some(Keyword::Xor),
            "not" => Some(Keyword::Not),
            _ => None
        }
    }

}