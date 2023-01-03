

////////////////////////////////////////////////////////////////////////////////////////////////////
//                                           Positioned                                           //
//////////////////////////////////////////////////////////////////////////////////////////////////// 
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Token {
    Keyword(Keyword),
    String(String)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Keyword {
    Use
}

impl Keyword {

    pub fn from_string(str: &str) -> Option<Keyword> {
        match str {
            "use" => Some(Keyword::Use),
            _ => None
        }
    }

}