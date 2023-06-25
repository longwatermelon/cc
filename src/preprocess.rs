use std::fs;

#[derive(Clone, PartialEq, Debug)]
struct Definition {
    name: String,
    expr: Option<String>
}

struct IfPair {
    if_expr: String,
    start: usize
}

pub struct Preprocessor {
    prog: String,
    defs: Vec<Vec<Definition>>,
    pending_ifs: Vec<IfPair>
}

impl Definition {
    fn new(name: String, expr: Option<String>) -> Self {
        Self { name, expr }
    }
}

impl IfPair {
    fn new(expr: String, start: usize) -> Self {
        Self { if_expr: expr, start }
    }
}

impl Preprocessor {
    pub fn new(prog: String) -> Self {
        Self { prog, defs: vec![Vec::new()], pending_ifs: Vec::new() }
    }

    pub fn preprocess(&mut self) {
        while self.prog.contains('#') {
            self.preprocess_once();
        }

        self.replace_defs();
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
                    "define" => return self.process_define(start, i),
                    "ifndef" => return self.process_ifndef(start, i),
                    "endif" => return self.process_endif(start, i),
                    _ => panic!()
                }
            }
        }
    }

    fn replace_defs(&mut self) {
        for layer in self.defs.clone() {
            for def in &layer {
                if let Some(expr) = &def.expr {
                    self.prog = self.prog.replace(def.name.as_str(), expr.as_str());
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

    fn process_define(&mut self, start: usize, mut index: usize) {
        while self.prog.chars().nth(index).unwrap().is_whitespace() {
            index += 1;
        }

        let mut id: String = String::new();
        while !self.prog.chars().nth(index).unwrap().is_whitespace() {
            id.push(self.prog.chars().nth(index).unwrap());
            index += 1;
        }

        let mut expr: String = String::new();
        if self.prog.chars().nth(index).unwrap() != '\n' {
            while self.prog.chars().nth(index).unwrap().is_whitespace() {
                index += 1;
            }

            while self.prog.chars().nth(index).unwrap() != '\n' {
                expr.push(self.prog.chars().nth(index).unwrap());
                index += 1;
            }
        }

        // self.defs is guaranteed to have last element
        self.defs.iter_mut().last().unwrap().push(Definition::new(id, if expr.is_empty() { None } else { Some(expr) }));
        self.prog.replace_range(start..index, "");
    }

    fn process_ifndef(&mut self, start: usize, mut index: usize) {
        self.defs.push(Vec::new());
        while self.prog.chars().nth(index).unwrap().is_whitespace() {
            index += 1;
        }

        let mut id: String = String::new();
        while !self.prog.chars().nth(index).unwrap().is_whitespace() {
            id.push(self.prog.chars().nth(index).unwrap());
            index += 1;
        }

        self.pending_ifs.push(IfPair::new(id, start));
        self.prog.replace_range(start..index, "");
    }

    fn process_endif(&mut self, start: usize, index: usize) {
        self.prog.replace_range(start..index, "");
        if let Some(last) = self.pending_ifs.last() {
            // For ifndef
            let mut exists: bool = false;
            for l in 0..self.defs.len() - 1 {
                for def in &self.defs[l] {
                    if def.name == last.if_expr {
                        exists = true;
                        break;
                    }
                }
            }

            if exists {
                self.prog.replace_range(last.start..start, "");
            }

            self.pending_ifs.pop();
            self.defs.pop();
        } else {
            eprintln!("preprocessing error: endif without if");
            std::process::exit(1);
        }
    }

    pub fn result(&self) -> String {
        self.prog.clone()
    }
}

