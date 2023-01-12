mod lexer;
mod parser;
mod errors;
mod types;
mod traceback;
mod repl;
mod environment;
mod native_functions;
mod functions;
mod resolver;
mod expression;
mod statement;

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
use resolver::Resolver;

pub struct Interpreter {
    environment: Env,
    output: Rc<RefCell<Box<dyn MyWrite>>>,
    resolver: Resolver,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Self::new_with_output(Rc::new(RefCell::new(Box::new(stdout()))))
    }

    pub fn new_with_output(output: Rc<RefCell<Box<dyn MyWrite>>>) -> Interpreter {
        let resolver = Resolver::new();
        let env = make_env();
        define_globals(&env);

        let res = Interpreter {
            environment: env,
            output,
            resolver
        };

        return res;
    }

    pub fn run_file(&mut self, path: &str) {
        if let Ok(mut file) = std::fs::File::open(path) {
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();

            if let Err(e) = self.run(contents) {
                print!("{}", e);
            }
        } else {
            println!("Could not open file {}", path);
        }
    }

    pub fn run_repl(&mut self) {
        let mut repl = Repl::new();
        
        while let Some(source) = repl.next() {
            self.environment.borrow_mut().set_env_var(EnvVariable::NewLines, DynValue::from(0));

            if let Err(result) = self.run(source.clone()) {
                repl.printerr(result);
            } else {
                let skip = self.environment.borrow().get_env_var(EnvVariable::NewLines).as_number() as u16;
                repl.skiplines(skip);
            }
        }
    }

    fn run(&mut self, source: String) -> Result<(), String> {
        if let Err(mut traceback) = self.run_with_traceback(source.clone()){
            traceback.code = Some(source);
            Err(report_trace(traceback))
        } else {
            Ok(())
        }
    }

    fn run_with_traceback(&mut self, source: String) -> Result<(), Traceback> {
        let mut lexer = Lexer::new(source);
        let mut parser = Parser::new(lexer.tokenize()?, self.output.clone());

        let program = parser.parse()?;

        for stmt in &program {
            stmt.resolve(&mut self.resolver)?;
        }

        self.environment.borrow_mut().set_resolved_locals(self.resolver.locals.clone());

        for stmt in &program {
            stmt.execute(&self.environment)?;
        }

        Ok(())
    }
}

// used in tests
pub fn run_to_string(source: String) -> String {
    let output = Rc::new(RefCell::new(Box::new(Vec::new()) as Box<dyn MyWrite>));
    let mut interpreter = Interpreter::new_with_output(output.clone());
    if let Err(errors) = interpreter.run(source.to_string()) {
        return errors;
    }
    return output.borrow().get_string().unwrap();
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

    fn test_run_case(test_case_name : &str, source: &str, expected: &str) {
        let output = run_to_string(source.to_string());

        assert_eq!(output.as_str(), expected, "Test case \"{}\" failed", test_case_name);
    }

    #[test]
    fn test_run() {
        test_run_case("simple print", "print 1", "1\n");

        test_run_case("simple math", "print 1 + 2", "3\n");

        test_run_case("quiet assignment", "a = 1", "");

        test_run_case("simple assignment", "a = 1\nprint a", "1\n");

        test_run_case("simple re-assignment", "a = 1\na = 2\nprint a", "2\n");

        test_run_case("simple foreach", r#"for a in [1,2,3]:
                         print(a)"#, "1\n2\n3\n");

        test_run_case("simple while",
                        r#"a=False
                         while a<10:
                           a=a+1
                         print(a)"#, "10\n");

        test_run_case("collatz of 27",
            r#"n=27
               i=0
               while n != 1:
                 if n%2==0:
                   n=n/2
                 else:
                   n=3*n+1
                 i=i+1
               print(i)"#, "111\n");

        test_run_case("simple function",
            "def hi():\n  print(\"hi\")\nhi()", "hi\n");

        test_run_case("simple 1-arg function",
            "def f(x):\n  print x+1\nf(1)", "2\n");

        test_run_case("simple multi-args function",
            "def f(x,y):\n  print x+y\nf(1,2)", "3\n");

test_run_case("simple return function",
"def add(a, b):
  return a + b
print(add(1, 2))
print(add(\"a\", \"b\"))
print(add(9.9, True))", "3\nab\n10.9\n");

test_run_case(
"fibonacci",
"def fib(n):
  if n < 2:
    return n
  return fib(n-1) + fib(n-2)
print(fib(10))", "55\n");

test_run_case(
"nested function",
"def f():
  def g():
    print(1)
  g()
f()", "1\n");

test_run_case(
"nested function, with closure environment and return function",
"def f():
  i=0
  def count():
    nonlocal i
    i=i+1
    print(i)
  return count
c = f()
c()
c()
c()", "1\n2\n3\n");

test_run_case(
"broken closure code",
"a=\"global\"
def f():
  def print_A():
    print(a)
  print_A()
  a=\"local\"
  print_A()
f()", "global\nglobal\n");
    }
}
