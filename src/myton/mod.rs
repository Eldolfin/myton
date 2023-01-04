mod lexer;
mod parser;
mod errors;

pub use errors::had_error;

use lexer::*;
use errors::{report_error, report_error_at};
use std::io::prelude::*;


pub fn run_file(path: &str) {
    todo!()
}

pub fn run_prompt(){
    println!("Myton 0.0.1 (main) [Rust 1.65.0] on linux");
    println!("Type \"help\" for more information.");
    loop {
        print!(">>> ");
        std::io::stdout().flush().unwrap();
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        run(input, "");
    }
}

fn run(source: String, file_name: &str) {
    let mut lexer = Lexer::new(source.clone());
    match lexer.tokenize() {
        Ok(tokens) => {
            for token in tokens.clone() {
                println!("{:?}", token);
            }
            match parser::parse(tokens) {
                Ok(expr) => {
                    // let res = expr.eval();
                    // println!("{}", res);
                },
                Err(e) => report_error(0, file_name, &e)
            }
        },
        Err(err) => {
            report_error_at(lexer.line_number, lexer.line_position, file_name, &err, &source);
        }
    }
}
