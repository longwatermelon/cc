mod error;
mod lexer;
mod node;
mod parser;

use error::Error;
use parser::Parser;
use node::Node;
use std::fs;

fn main() {
    let prog: String = fs::read_to_string("examples/test.c").expect("Couldn't read file examples/test.c.");

    let mut parser: Parser = Parser::new(prog).unwrap();
    let root: Result<Node, Error> = parser.parse();

    match root {
        Ok(x) => println!("{:#?}", x),
        Err(e) => println!("{}", e.to_string())
    }
}

