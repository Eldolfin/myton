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
use parser::Parser;
use traceback::Traceback;
use repl::Repl;
use environment::{Env,make_env, EnvVariable};
use std::io::prelude::*;
use native_functions::define_globals;
use types::DynValue;
use std::rc::Rc;
use std::cell::RefCell;
use std::io::{Write, Stdout, stdout};

pub struct Interpreter {
    environment: Env,
    output: Rc<RefCell<Box<dyn MyWrite>>>,
}

const DEBUG_LEXER: bool = false;

impl Interpreter {
    pub fn new() -> Interpreter {
        Self::new_with_output(Rc::new(RefCell::new(Box::new(stdout()))))
    }

    pub fn new_with_output(output: Rc<RefCell<Box<dyn MyWrite>>>) -> Interpreter {
        let env = make_env();
        define_globals(&env);

        Interpreter {
            environment: env,
            output,
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
        
        while let Some(source) = repl.next() {
            self.environment.borrow_mut().set_env_var(EnvVariable::NewLines, DynValue::from(0));
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
            traceback.code = Some(source.lines().nth(traceback.pos.1)
                .unwrap_or(format!("Could not find line {} in source", traceback.pos.1).as_str()).to_string());
            Err(report_trace(traceback))
        } else {
            Ok(())
        }
    }

    fn run_with_traceback(&mut self, source: String) -> Result<(), Traceback> {
        let mut lexer = Lexer::new(source);
        let mut parser = Parser::new(lexer.tokenize()?, self.output.clone());

        let program = parser.parse()?;

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




pub trait MyWrite : Write {
    fn get_string(&self) -> Option<String>;
}

impl MyWrite for Vec<u8> {
    fn get_string(&self) -> Option<String> {
        Some(String::from_utf8(self.clone()).unwrap())
    }
}

impl MyWrite for Stdout {
    fn get_string(&self) -> Option<String> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_run_case(source: &str, expected: &str) {
        let output = Rc::new(RefCell::new(Box::new(Vec::new()) as Box<dyn MyWrite>));
        let mut interpreter = Interpreter::new_with_output(output.clone());
        interpreter.run(source.to_string()).unwrap();

        assert_eq!(output.borrow().get_string().unwrap().as_str(), expected);
    }

    #[test]
    fn test_run() {
        test_run_case("print 1", "1\n");
        test_run_case("print 1 + 1", "2\n");
        test_run_case("a=1", "");
        test_run_case("a=1\nprint a", "1\n");
        test_run_case("a=1\na=2\nprint a", "2\n");
        test_run_case("a=1\nprint a\na=2\nprint a", "1\n2\n");
        test_run_case("a=1\nprint(a)", "1\n");
        test_run_case("for a in [1,2,3]:\n  print(a)", "1\n2\n3\n");
        test_run_case("a=False\nwhile a<10:\n  a=a+1\nprint(a)", "10\n");
        test_run_case(
"n=27
i=0
while n != 1:
  if n%2==0:
    n=n/2
  else:
    n=3*n+1
  i=i+1
print(i)", "111\n");

    }
}
