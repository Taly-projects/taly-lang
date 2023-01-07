use colored::{ColoredString, Colorize};

use crate::util::{position::Positioned, source_file::SourceFile};

////////////////////////////////////////////////////////////////////////////////////////////////////
//                                           Error Type                                           //
//////////////////////////////////////////////////////////////////////////////////////////////////// 

pub enum ErrorType {
    Error,
    Warning
}



////////////////////////////////////////////////////////////////////////////////////////////////////
//                                          Error Format                                          //
//////////////////////////////////////////////////////////////////////////////////////////////////// 

pub struct ErrorFormat {
    error_type: ErrorType,
    pos: Option<Positioned<()>>,
    message: String,
    step: String,
}

impl ErrorFormat {

    pub fn new(error_type: ErrorType) -> Self {
        Self {
            error_type,
            pos: None,
            message: "No Message".to_string(),
            step: "No Step".to_string()
        }
    }

    pub fn set_pos(mut self, pos: Positioned<()>) -> Self {
        self.pos = Some(pos);
        self
    }

    pub fn set_message(mut self, msg: String) -> Self {
        self.message = msg;
        self
    }

    pub fn set_step(mut self, step: String) -> Self {
        self.step = step;
        self
    }

    fn color_msg(&self, str: String) -> ColoredString {
        match self.error_type {
            ErrorType::Error => str.truecolor(255, 81, 81),
            ErrorType::Warning => str.truecolor(255, 255, 81),
        }
    }

    pub fn print(self, src: &SourceFile) {
        println!("{} {}", self.color_msg(format!("[{}]:", self.step)).bold(), self.message);
        print!("      {} in {}", "=>".truecolor(81, 81, 255).bold(), src.name_ext());
        if let Some(pos) = self.pos.clone() {
            println!(":{}:{}", pos.start.line, pos.start.column_index + 1);
            println!("       {}", "|".truecolor(81, 81, 255).bold());
            
            let mut lines = src.src.lines();
            let mut line = pos.start.line;

            let mut current_line = lines.nth(line - 1).unwrap();
            while line <= pos.end.line {
                let space_offset = (line == pos.start.line).then_some(pos.start.column_index).unwrap_or(0);
                let error_length = (line == pos.end.line).then_some(pos.end.column_index - space_offset).unwrap_or(current_line.len() - space_offset);
            
                println!(" {:>5} {} {}", line.to_string().truecolor(81, 81, 255).bold(), "|".truecolor(81, 81, 255).bold(), current_line);
                print!("       {}", "|".truecolor(81, 81, 255).bold());
                println!(" {}{}", " ".repeat(space_offset), self.color_msg("^".repeat(error_length).to_string()).bold());

                line += 1;
                if let Some(l) = lines.next() {
                    current_line = l;
                } else {
                    break;
                }
            }
        } else {
            println!();
        }
    }

}