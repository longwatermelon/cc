use std::fs;

struct Definition {
    name: String,
    expr: Option<String>
}

pub struct Preprocessor {
    prog: String,
    defs: Vec<Definition>
}

impl Definition {
    fn new(name: String, expr: Option<String>) -> Self {
        Self { name, expr }
    }
}

impl Preprocessor {
    pub fn new(prog: String) -> Self {
        Self { prog, defs: Vec::new() }
    }

    pub fn preprocess(&mut self) {
        while self.prog.contains('#') {
            self.preprocess_once();
        }
    }

    pub fn result(&self) -> String {
        self.prog.clone()
    }

    fn preprocess_once(&mut self) {
        for mut i in 0..self.prog.len() {
            let start: usize = i;
            if self.prog.chars().nth(i).unwrap() == '#' {
                i += 1;
                let mut cmd: String = String::new();
                while !self.prog.chars().nth(i).unwrap().is_whitespace() {
                    cmd += self.prog.chars().nth(i).unwrap().to_string().as_str();
                    i += 1;
                }

                match cmd.as_str() {
                    "include" => return self.process_include(start, i),
                    _ => panic!()
                }
            }
        }
    }

    fn process_include(&mut self, start: usize, mut index: usize) {
        while self.prog.chars().nth(index).unwrap() != '"' {
            index += 1;
        }
        index += 1;

        let mut path: String = String::new();
        while self.prog.chars().nth(index).unwrap() != '"' {
            path.push(self.prog.chars().nth(index).unwrap());
            index += 1;
        }
        index += 1;

        self.prog.replace_range(start..index, fs::read_to_string(path.as_str()).unwrap().as_str());
    }
}

