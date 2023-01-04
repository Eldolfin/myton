mod lexer;
mod parser;
mod errors;
mod types;
mod traceback;
mod repl;

pub use errors::had_error;

use lexer::*;
use errors::report_trace;
use parser::parse;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use std::io::{stdout, stdin, Write};
use types::DynValue;
use traceback::Traceback;
use repl::Repl;

pub fn run_file(path: &str) {
    todo!()
}

pub fn run_repl() {
    Repl::new().run();
}

fn run(source: String) -> Result<(), Traceback> {
    let mut lexer = Lexer::new(source.clone());

    let program = parse(lexer.tokenize()?)?;

    for stmt in program {
        let res = stmt.execute();
        res?;
    }

    Ok(())
}
