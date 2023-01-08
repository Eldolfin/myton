mod lexer;
mod parser;
mod errors;
mod types;
mod traceback;
mod repl;
mod environment;
mod native_functions;
mod functions;

pub use errors::had_error;

use lexer::*;
use errors::report_trace;
use parser::parse;
use traceback::Traceback;
use repl::Repl;
use environment::{Env,make_env, EnvVariable};
use std::io::prelude::*;
use native_functions::define_globals;
use types::DynValue;

pub struct Interpreter {
    environment: Env,
}

const DEBUG_LEXER: bool = false;

impl Interpreter {
    pub fn new() -> Interpreter {
        let env = make_env();
        define_globals(&env);

        Interpreter {
            environment: env,
        }
    }

    pub fn run_file(&mut self, path: &str) {
        if let Ok(mut file) = std::fs::File::open(path) {
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();
            
            self.debug_lexer(contents.clone());

            if let Err(e) = self.run(contents) {
                print!("\x1b[31m{}\x1b[0m", e);
            }
        } else {
            println!("Could not open file {}", path);
        }
    }

    pub fn run_repl(&mut self) {
        let mut repl = Repl::new();
        repl.welcome_prompt();
        self.environment.borrow_mut().set_env_var(EnvVariable::NewLines, DynValue::from(0));
        
        while let Some(source) = repl.next() {
            self.debug_lexer(source.clone());

            if let Err(result) = self.run(source.clone()) {
                repl.printerr(result);
            } else {
                let skip = self.environment.borrow().get_env_var(EnvVariable::NewLines).as_number() as u16;
                repl.skiplines(skip);
            }
        }
    }

    fn format_tokens(&self, source: String) -> String {
        let mut s = String::new();
        if let Ok(tokens) = Lexer::new(source).tokenize(){
            for token in tokens {
                s.push_str(&format!("{:?}\n", token));
            }
        }
        s
    }

    fn run(&mut self, source: String) -> Result<(), String> {
        if let Err(mut traceback) = self.run_with_traceback(source.clone()){
            traceback.code = Some(source.lines().nth(traceback.pos.1).unwrap().to_string());
            Err(report_trace(traceback))
        } else {
            Ok(())
        }
    }

    fn run_with_traceback(&mut self, source: String) -> Result<(), Traceback> {
        let mut lexer = Lexer::new(source);

        let program = parse(lexer.tokenize()?)?;

        for stmt in program {
            stmt.execute(&self.environment)?;
        }

        Ok(())
    }

    fn debug_lexer(&self, contents: String) {
        if DEBUG_LEXER {
            println!("{}", self.format_tokens(contents));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_run_case(source: &str, expected: &str) {
        let mut interpreter = Interpreter::new();
        match interpreter.run(source.to_string()){
            Err(e) => {
                print!("{}", e);
                panic!("Error running test case");
            }
            _ => {}
        }
    }

//    #[test]
//    fn test_run() {
//        test_run_case("print 1", "1\n");
//        test_run_case("print 1 + 1", "2\n");
//        test_run_case("a=1", "");
//        test_run_case("a=1\nprint a", "1\n");
//        test_run_case("a=1\na=2\nprint a", "2\n");
//        test_run_case("a=1\nprint a\na=2\nprint a", "1\n2\n");
//        test_run_case("a=1\nprint(a)", "1\n");
//        test_run_case("for a in [1,2,3]:\n  print(a)", "1\n2\n3\n");
//        test_run_case("a=False\nwhile a<10:\n  a=a+1\nprint(a)", "10\n");
//        test_run_case(
//"n=27
//i=0
//while n != 1:
//  if n%2==0:
//    n=n/2
//  else:
//    n=3*n+1
//  i=i+1
//print(i)", "111\n");

//    }
}
