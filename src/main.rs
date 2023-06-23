mod error;
mod lexer;
mod node;
mod parser;

use parser::Parser;
use node::Node;
use std::fs;

fn main() {
    let prog: String = fs::read_to_string("examples/test.c").expect("Couldn't read file examples/test.c.");

    let mut parser: Parser = Parser::new(prog).unwrap();
    let root: Node = parser.parse().unwrap();
    println!("{:#?}", root);
}

