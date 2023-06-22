use colored::Colorize;
use std::fmt;

pub struct Error {
    message: String,
    line: usize
}

impl Error {
    pub fn new(message: String, line: usize) -> Self {
        Self {
            message, line
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: Line {}: {}", "error".bright_red(), self.line, self.message)
    }
}

