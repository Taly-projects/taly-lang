use std::fmt::Debug;

////////////////////////////////////////////////////////////////////////////////////////////////////
//                                            Position                                            //
//////////////////////////////////////////////////////////////////////////////////////////////////// 

#[derive(Clone)]
pub struct Position {
    pub index: usize,
    pub line: usize,
    pub column: usize,
    pub column_index: usize
}

impl Default for Position {
    
    fn default() -> Self {
        Self { 
            index: 0,
            line: 1,
            column: 0,
            column_index: 0
        }
    }

}

impl Position {

    pub fn advance(&mut self, chr: char) {
        if chr == '\n' {
            self.line += 1;
            self.column = 0;
            self.column_index = 0;
        } else if chr == '\t' {
            self.column += 4;
            self.column_index += 1;
        } else {
            self.column += 1;
            self.column_index += 1;
        }
        self.index += 1;
    }

}

////////////////////////////////////////////////////////////////////////////////////////////////////
//                                           Positioned                                           //
//////////////////////////////////////////////////////////////////////////////////////////////////// 
#[derive(Clone)]
pub struct Positioned<T> {
    pub data: T,
    pub start: Position,
    pub end: Position
}

impl<T> Positioned<T> {

    pub fn new(data: T, start: Position, end: Position) -> Self {
        Self {
            data,
            start,
            end
        }
    } 

    pub fn convert<U>(&self, data: U) -> Positioned<U> {
        Positioned {
            data,
            start: self.start.clone(),
            end: self.end.clone()
        }
    }

}

impl<T: Debug> Debug for Positioned<T> {
    
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.data.fmt(f)
    }

}