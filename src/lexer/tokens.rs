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
    DoubleEqual,
    ExclamationMarkEqual,
    Plus,
    Dash,
    Star,
    Slash,
    Dot,
    LeftAngle,
    LeftAngleEqual,
    RightAngle,
    RightAngleEqual,
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
    Not,
    If,
    Elif,
    Else,
    Then,
    End,
    While,
    Do
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
            "if" => Some(Keyword::If),
            "elif" => Some(Keyword::Elif),
            "else" => Some(Keyword::Else),
            "then" => Some(Keyword::Then),
            "end" => Some(Keyword::End),
            "while" => Some(Keyword::While),
            "do" => Some(Keyword::Do),
            _ => None
        }
    }

}