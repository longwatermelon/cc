use std::fs;

pub fn preprocess(prog: String) -> String {
    for mut i in 0..prog.len() {
        let start: usize = i;
        if prog.chars().nth(i).unwrap() == '#' {
            i += 1;
            let mut cmd: String = String::new();
            while !prog.chars().nth(i).unwrap().is_whitespace() {
                cmd += prog.chars().nth(i).unwrap().to_string().as_str();
                i += 1;
            }

            match cmd.as_str() {
                "include" => return process_include(prog.clone(), start, i),
                _ => panic!()
            }
        }
    }

    prog
}

fn process_include(mut prog: String, start: usize, mut index: usize) -> String {
    while prog.chars().nth(index).unwrap() != '"' {
        index += 1;
    }
    index += 1;

    let mut path: String = String::new();
    while prog.chars().nth(index).unwrap() != '"' {
        path.push(prog.chars().nth(index).unwrap());
        index += 1;
    }
    index += 1;

    prog.replace_range(start..index, fs::read_to_string(path.as_str()).unwrap().as_str());
    prog
}

