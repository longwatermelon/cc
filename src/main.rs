mod error;
mod lexer;
mod node;
mod parser;
mod preprocess;
mod cdefs;
mod scope;
mod asm;

use error::Error;
use parser::Parser;
use node::Node;
use preprocess::Preprocessor;
use asm::Gen;
use std::fs;
use std::io::Write;
use std::process::Command;

fn handle_err<T>(res: Result<T, Error>, prog: &str) -> T {
    if let Err(e) = res {
        e.print(prog);
        std::process::exit(1);
    } else {
        res.unwrap()
    }
}

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("No input files provided.");
        std::process::exit(1);
    }

    // Preprocessor
    let prog: String = match fs::read_to_string(&args[0]) {
        Ok(x) => x,
        Err(_) => {
            eprintln!("Can't read file '{}'.", args[0]);
            std::process::exit(1);
        }
    };

    let mut preprocessor: Preprocessor = Preprocessor::new(&prog);
    preprocessor.preprocess();
    let processed: String = preprocessor.result();

    // Parser
    let mut parser: Parser = Parser::new(&processed).unwrap();
    let root: Node = handle_err(parser.parse(), &prog);

    // Assembly generation
    let mut generator: Gen = Gen::new();
    let result: String = handle_err(generator.gen(&root), &prog);

    // Write to file
    let mut f = fs::File::create("a.s").expect("Unable to create file 'a.s'.");
    f.write_all(result.as_bytes()).expect("Unable to write to file 'a.s'.");

    // Assemble
    let output = Command::new("sh").args(["-c", "nasm -felf64 a.s && ld *.o && rm *.o"]).output().unwrap();
    if !output.status.success() {
        println!("{:#?}", output);
    }

    Ok(())
}

