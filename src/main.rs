mod myton;
use crate::myton::{run_file, run_prompt};
use std::env::args;

fn main(){
    let args: Vec<String> = args().collect();
    if args.len() > 2 {
        println!("Usage: myton [script]");
        std::process::exit(64);
    } else if args.len() == 2 {
        run_file(&args[1]);
    } else {
        run_prompt();
    }

    if myton::had_error() {
        std::process::exit(65);
    }
}
