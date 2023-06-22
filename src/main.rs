mod error;
mod lexer;

use lexer::{Lexer, Token, TokenType};
use std::fs;

fn main() {
    let prog: String = fs::read_to_string("examples/test.c").expect("Couldn't read file examples/test.c.");
    let mut lexer: Lexer = Lexer::new(prog);

    loop {
        let tok: Token = lexer.next().unwrap();
        println!("{:?} {}", tok.ttype, tok.value);

        if tok.ttype == TokenType::Eof {
            break;
        }
    }
}

