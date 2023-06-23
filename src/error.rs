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

    pub fn to_string(&self) -> String {
        format!("{}: Line {}: {}", "error".bright_red(), self.line, self.message)
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

