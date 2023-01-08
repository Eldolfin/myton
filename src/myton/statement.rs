use std::rc::Rc;
use std::cell::RefCell;
use std::io::Write;

use super::environment::{Env, EnvVariable};
use super::MyWrite;
use super::expression::Expr;
use super::traceback::Traceback;
use super::token::Token;
use super::types::DynValue;
use super::functions::Function;

pub trait Statement {
    fn execute(&self, env: &Env) -> Result<(), Traceback>;
}

pub type StmtTrait = Box<dyn Statement>;
pub type STMT = Box<dyn Statement>;

pub struct ExpressionStatement {
    pub expression: Expr,
}

pub struct IfStatement {
    pub condition: Expr,
    pub then_branch: STMT,
    pub else_branch: Option<STMT>,
}

pub struct WhileStatement {
    pub condition: Expr,
    pub body: STMT,
}

pub struct ForeachStatement {
    pub variable: String,
    pub collection: Expr,
    pub body: STMT,
}


pub struct PrintStatement {
    pub expression: Expr,
    pub output: Rc<RefCell<Box<dyn MyWrite>>>,
}

pub struct VarStatement {
    pub name: String,
    pub initializer: Expr,
}

pub struct BlockStatement {
    pub statements: Vec<STMT>,
}

pub struct FunctionStatementInner {
    pub name: String,
    pub parameters: Vec<String>,
    pub body: STMT,
}

pub struct FunctionStatement {
    pub inner: Rc<RefCell<FunctionStatementInner>>
}

pub struct ReturnStatement {
    pub keyword: Token,
    pub value: Option<Expr>,
}

impl Statement for ExpressionStatement {
    fn execute(&self, env: &Env) -> Result<(), Traceback> {
        self.expression.eval(env)?;
        Ok(())
    }
}

impl Statement for IfStatement {
    fn execute(&self, env: &Env) -> Result<(), Traceback> {
        if self.condition.eval(env)?.as_bool() {
            self.then_branch.execute(env)
        } else if let Some(else_branch) = &self.else_branch {
            else_branch.execute(env)
        } else {
            Ok(())
        }
    }
}

impl Statement for PrintStatement {
    fn execute(&self, env: &Env) -> Result<(), Traceback> {
        let value = self.expression.eval(env)?.as_string();
        
        let line_nb = value.lines().count();
        env.borrow().get_env_var(EnvVariable::NewLines).increment_by(line_nb as f64);
        
        writeln!(self.output.borrow_mut(), "{}", value).unwrap();
        
        Ok(())
    }
}

impl Statement for VarStatement {
    fn execute(&self, env: &Env) -> Result<(), Traceback> {
        let value = self.initializer.eval(env)?;

        env.borrow_mut().set(self.name.clone(), value);

        Ok(())
    }
}

impl Statement for BlockStatement {
    fn execute(&self, env: &Env) -> Result<(), Traceback> {
        for statement in &self.statements {
            statement.execute(env)?;
        }
        Ok(())
    }
}

impl Statement for WhileStatement {
    fn execute(&self, env: &Env) -> Result<(), Traceback> {
        while self.condition.eval(env)?.as_bool() {
            self.body.execute(env)?;
        }
        Ok(())
    }
}

impl Statement for ForeachStatement {
    fn execute(&self, env: &Env) -> Result<(), Traceback> {
        let list = self.collection.eval(env)?;
        if let Some(array) = list.as_list() {
            for value in array {
                env.borrow_mut().set(self.variable.clone(), value);
                self.body.execute(env)?;
            }
            Ok(())
        } else {
            Err(Traceback {
                message: Some(format!("'{}' object is not iterable", list.tipe)),
                ..Default::default()})
        }
    }
}

impl Statement for FunctionStatement {
    fn execute(&self, env: &Env) -> Result<(), Traceback> {
        let name = self.inner.borrow().name.clone();

        let function = DynValue::from_function(Function::new(self.clone(), env.clone()), name.clone());

        env.borrow_mut().set(name, function);

        Ok(())
    }
}

impl FunctionStatement {
    pub fn new(name: String, parameters: Vec<String>, body: STMT) -> Self {
        Self {
            inner: Rc::new(RefCell::new(FunctionStatementInner {
                name,
                parameters,
                body,
            })),
        }
    }
}


impl Statement for ReturnStatement {
    fn execute(&self, env: &Env) -> Result<(), Traceback> {
        Err(Traceback::from_return_value(
            if let Some(expr) = &self.value {
                expr.eval(env)?
            } else {
                DynValue::none()
            }
        ))
    }
}

impl Clone for FunctionStatement {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

