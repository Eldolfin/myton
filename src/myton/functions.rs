use std::cell::RefCell;
use std::rc::Rc;

use super::class::Instance;
use super::environment::{make_env_enclosed, Env};
use super::statement::FunctionStatement;
use super::traceback::{Traceback, TracebackKind};
use super::types::DynValue;

pub trait Callable {
    fn call(&self, env: &Env, args: Vec<DynValue>) -> Result<DynValue, Traceback>;

    fn arity(&self) -> usize;
}

#[derive(Clone)]
pub struct Function {
    pub statement: FunctionStatement,
    pub closure: Env,
}

#[derive(Clone)]
pub struct NativeFunction {
    pub func: fn(&Env, Vec<DynValue>) -> Result<DynValue, Traceback>,
    pub nb_args: usize,
}

impl Function {
    pub fn new(statement: FunctionStatement, closure: Env) -> Self {
        Self { statement, closure }
    }

    pub fn bind(&self, instance: Rc<RefCell<Instance>>) -> Self {
        let env = make_env_enclosed(self.closure.clone());
        env.borrow_mut()
            .set("this".to_string(), DynValue::from(instance));
        Self {
            statement: self.statement.clone(),
            closure: env,
        }
    }
}

impl Callable for Function {
    fn call(&self, _: &Env, args: Vec<DynValue>) -> Result<DynValue, Traceback> {
        let function_env = make_env_enclosed(self.closure.clone());

        for (param, value) in self
            .statement
            .inner
            .as_ref()
            .borrow()
            .parameters
            .iter()
            .zip(args)
        {
            function_env.borrow_mut().set(param.value.clone(), value);
        }

        match self
            .statement
            .inner
            .as_ref()
            .borrow()
            .body
            .execute(&function_env)
        {
            Err(Traceback {
                tipe: TracebackKind::Return,
                value: Some(value),
                ..
            }) => Ok(value),
            Ok(()) => Ok(DynValue::none()),
            Err(traceback) => Err(traceback),
        }
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
