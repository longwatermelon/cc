mod error;
mod lexer;
mod node;
mod parser;

use error::Error;
use parser::Parser;
use node::Node;
use std::fs;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("no input files provided.");
        std::process::exit(1);
    }

    let prog: String = fs::read_to_string(args[0].as_str()).expect("Couldn't read file examples/test.c.");

    let mut parser: Parser = Parser::new(prog).unwrap();
    let root: Result<Node, Error> = parser.parse();

    match root {
        Ok(x) => println!("{:#?}", x),
        Err(e) => println!("{}", e)
    }
}

