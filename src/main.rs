mod error;
mod lexer;
mod node;
mod parser;
mod preprocess;
mod gen;
mod scope;

use error::Error;
use parser::Parser;
use node::Node;
use preprocess::Preprocessor;
use gen::Gen;
use std::fs;
use std::io::Write;
use std::process::Command;

fn handle_err<T>(res: Result<T, Error>) -> T {
    if let Err(e) = res {
        eprintln!("{}", e);
        std::process::exit(1);
    } else {
        res.unwrap()
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("no input files provided.");
        std::process::exit(1);
    }

    // Preprocessor
    let prog: String = fs::read_to_string(args[0].as_str()).expect("Couldn't read file examples/test.c.");
    let mut preprocessor: Preprocessor = Preprocessor::new(prog);
    preprocessor.preprocess();
    let processed: String = preprocessor.result();

    // Parser
    let mut parser: Parser = Parser::new(processed).unwrap();
    let root: Node = handle_err(parser.parse());

    // Assembly generation
    let mut generator: Gen = Gen::new();
    let result: String = handle_err(generator.gen(&root));

    // Write to file
    let mut f = fs::File::create("a.s").expect("Unable to create file 'a.s'.");
    f.write_all(result.as_bytes()).expect("Unable to write to file 'a.s'.");

    // Assemble
    Command::new("sh").args(&["-c", "nasm -felf64 a.s && ld *.o && rm *.o"]).output().unwrap();
}

