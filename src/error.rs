use colored::Colorize;

#[derive(Debug)]
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

    pub fn print(&self, prog: &str) {
        let split: Vec<&str> = prog.split('\n').collect();
        println!("{}: Line {}: {}", "error".bright_red(), self.line, self.message);
        let longest: usize = *[self.line - 1, self.line, self.line + 1].map(|x| x.to_string().len()).iter().max().unwrap();
        for i in -1i32..=1 {
            if (i + self.line as i32) < 0 || (i + self.line as i32) >= split.len() as i32 {
                continue;
            }

            let padding: usize = longest - (self.line as i32 + i).to_string().len();
            let format: String = format!("  {}{} | {}", self.line as i32 + i, " ".repeat(padding), split[(self.line as i32 + i - 1) as usize]);
            println!("{}", if i == 0 { format.white().bold() } else { format.truecolor(150, 150, 150) });
        }
    }
}

