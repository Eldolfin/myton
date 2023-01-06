mod myton;

use crate::myton::Interpreter;
use std::env::args;

fn main(){
    let args: Vec<String> = args().collect();
    let mut myton = Interpreter::new();

    if args.len() > 2 {
        println!("Usage: myton [script]");
        std::process::exit(64);
    } else if args.len() == 2 {
        myton.run_file(&args[1]);
    } else {
        myton.run_repl();
    }

    if myton::had_error() {
        std::process::exit(65);
    }
}
