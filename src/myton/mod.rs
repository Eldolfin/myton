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
use environment::{Env,make_env};
use std::io::prelude::*;
use native_functions::add_native_functions;

pub struct Interpreter {
    environment: Env,
}

const DEBUG_LEXER: bool = true;

impl Interpreter {
    pub fn new() -> Interpreter {
        let env = make_env();
        add_native_functions(&env);

        Interpreter {
            environment: env,
        }
    }

    pub fn run_file(&mut self, path: &str) {
        if let Ok(mut file) = std::fs::File::open(path) {
            let mut contents = String::new();

            file.read_to_string(&mut contents).unwrap();
            
            if DEBUG_LEXER {
                println!("{}", self.format_tokens(contents.clone()));
            }


            match self.run(contents) {
                Ok(s) => print!("{}", s),
                Err(e) => print!("\x1b[31m{}\x1b[0m", e),
            }
        } else {
            println!("Could not open file {}", path);
        }
    }

    pub fn run_repl(&mut self) {
        let mut repl = Repl::new();
        repl.welcome_prompt();
        
        while let Some(source) = repl.next() {
            let result = self.run(source.clone());

            if DEBUG_LEXER {
                repl.println(self.format_tokens(source));
            }

            repl.print_result(result);
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

    fn run(&mut self, source: String) -> Result<String, String> {
        match self.run_with_traceback(source.clone()) {
            Ok(value) => Ok(value.to_string()),
            Err(traceback) => {
                let mut traceback = traceback.clone();
                traceback.code = Some(source.lines().nth(traceback.pos.1).unwrap().to_string());
                Err(report_trace(traceback))
            },
        }
    }

    fn run_with_traceback(&mut self, source: String) -> Result<String, Traceback> {
        let mut lexer = Lexer::new(source);

        let program = parse(lexer.tokenize()?)?;

        let mut results = String::new();

        for mut stmt in program {
            let r = stmt.execute(&self.environment)?;

            results.push_str(&r);
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_run_case(source: &str, expected: &str) {
        let mut interpreter = Interpreter::new();
        match interpreter.run(source.to_string()){
            Ok(result) => assert_eq!(result, expected),
            Err(e) => {
                print!("{}", e);
                panic!("Error running test case");
            }
        }
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
