use super::types::DynValue;
use super::traceback::Traceback;
use super::environment::{Env, make_env_enclosed};
use super::parser::FunctionStatement;


pub trait Callable {
    fn call(&self, env: &Env, args: Vec<DynValue>) -> Result<DynValue, Traceback>;

    fn arity(&self) -> usize;
}

pub struct Function {
    pub statement: FunctionStatement,
}

pub struct NativeFunction {
    pub func: fn(&Env, Vec<DynValue>) -> Result<DynValue, Traceback>,
    pub nb_args: usize,
}

impl Function {
    pub fn new(statement: FunctionStatement) -> Function {
        Function {
            statement,
        }
    }
}

impl Callable for Function {
    fn call(&self, env: &Env, args: Vec<DynValue>) -> Result<DynValue, Traceback> {
        let function_env = make_env_enclosed(env.clone());

        for (param, value) in self.statement.inner.as_ref().borrow().parameters.iter().zip(args) {
            function_env.borrow_mut().set(param.clone(), value);
        }

        self.statement.inner.as_ref().borrow().body.execute(&function_env)?;
        Ok(DynValue::none())
    }

    fn arity(&self) -> usize {
        self.statement.inner.as_ref().borrow().parameters.len()
    }
}

impl Callable for NativeFunction {
    fn call(&self, env: &Env, args: Vec<DynValue>) -> Result<DynValue, Traceback> {
        (self.func)(env, args)
    }

    fn arity(&self) -> usize {
        self.nb_args
    }
}

impl Clone for Function {
    fn clone(&self) -> Self {
        Function {
            statement: self.statement.clone(),
        }
    }
}

impl Clone for NativeFunction {
    fn clone(&self) -> Self {
        NativeFunction {
            func: self.func,
            nb_args: self.nb_args,
        }
    }
}
