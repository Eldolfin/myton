use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::io::Write;

use super::environment::{Env, EnvVariable};
use super::MyWrite;
use super::expression::EXPR;
use super::traceback::Traceback;
use super::token::Token;
use super::types::DynValue;
use super::functions::Function;
use super::resolver::Resolvable;
use super::class::Class;

pub trait Executable {
    fn execute(&self, env: &Env) -> Result<(), Traceback>;
}

pub trait Statement: Executable + Resolvable {}

pub type STMT = Box<dyn Statement>;

pub struct ExpressionStatement {
    pub expression: EXPR,
}

pub struct IfStatement {
    pub condition: EXPR,
    pub then_branch: STMT,
    pub else_branch: Option<STMT>,
}

pub struct WhileStatement {
    pub condition: EXPR,
    pub body: STMT,
}

pub struct ForeachStatement {
    pub variable: Token,
    pub collection: EXPR,
    pub body: STMT,
}


pub struct PrintStatement {
    pub expression: EXPR,
    pub output: Rc<RefCell<Box<dyn MyWrite>>>,
}

pub struct VarStatement {
    pub name: Token,
    pub initializer: EXPR,
}

pub struct BlockStatement {
    pub statements: Vec<STMT>,
}

pub struct FunctionStatementInner {
    pub name: Token,
    pub parameters: Vec<Token>,
    pub body: STMT,
}

pub struct FunctionStatement {
    pub inner: Rc<RefCell<FunctionStatementInner>>
}

pub struct ReturnStatement {
    pub keyword: Token,
    pub value: Option<EXPR>,
}

pub struct GlobalStatement {
    pub names: Vec<Token>,
}

pub struct NonlocalStatement {
    pub names: Vec<Token>,
}

pub struct ClassStatement {
    pub name: Token,
    pub methods: Vec<FunctionStatement>,
}

impl Executable for ExpressionStatement {
    fn execute(&self, env: &Env) -> Result<(), Traceback> {
        self.expression.eval(env)?;
        Ok(())
    }
}

impl Executable for IfStatement {
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

impl Executable for PrintStatement {
    fn execute(&self, env: &Env) -> Result<(), Traceback> {
        let value = self.expression.eval(env)?.as_string();
        
        let line_nb = value.lines().count();
        env.borrow().get_env_var(EnvVariable::NewLines).increment_by(line_nb as f64);
        
        writeln!(self.output.borrow_mut(), "{}", value).unwrap();
        
        Ok(())
    }
}

impl Executable for VarStatement {
    fn execute(&self, env: &Env) -> Result<(), Traceback> {
        let value = self.initializer.eval(env)?;

        env.borrow_mut().set(self.name.value.clone(), value);

        Ok(())
    }
}

impl Executable for BlockStatement {
    fn execute(&self, env: &Env) -> Result<(), Traceback> {
        for statement in &self.statements {
            statement.execute(env)?;
        }
        Ok(())
    }
}

impl Executable for WhileStatement {
    fn execute(&self, env: &Env) -> Result<(), Traceback> {
        while self.condition.eval(env)?.as_bool() {
            self.body.execute(env)?;
        }
        Ok(())
    }
}

impl Executable for ForeachStatement {
    fn execute(&self, env: &Env) -> Result<(), Traceback> {
        let list = self.collection.eval(env)?;
        if let Some(array) = list.as_list() {
            for value in array {
                env.borrow_mut().set(self.variable.value.clone(), value);
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

impl Executable for FunctionStatement {
    fn execute(&self, env: &Env) -> Result<(), Traceback> {
        let name = self.inner.borrow().name.clone();

        let function = DynValue::from_function(Function::new(self.clone(), env.clone()), name.value.clone());

        env.borrow_mut().set(name.value, function);

        Ok(())
    }
}

impl FunctionStatement {
    pub fn new(name: Token, parameters: Vec<Token>, body: STMT) -> Self {
        Self {
            inner: Rc::new(RefCell::new(FunctionStatementInner {
                name,
                parameters,
                body,
            })),
        }
    }
}


impl Executable for ReturnStatement {
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

impl Executable for GlobalStatement {
    fn execute(&self, env: &Env) -> Result<(), Traceback> {
        for name in &self.names {
            env.borrow_mut().set_global(name.value.clone());
        }
        Ok(())
    }
}

impl Executable for NonlocalStatement {
    fn execute(&self, env: &Env) -> Result<(), Traceback> {
        for name in &self.names {
            env.borrow_mut().set_nonlocal(name.value.clone());
        }
        Ok(())
    }
}

impl Executable for ClassStatement {
   fn execute(&self, env: &Env) -> Result<(), Traceback> {

        let methods : HashMap<String, Function> = self.methods.iter().map(|method| {
            let name = method.inner.borrow().name.clone();
            let function = Function::new(method.clone(), env.clone());
            (name.value, function)
        }).collect();
        
        let class = Class::new(self.name.value.clone(), methods);

        env.borrow_mut().set(self.name.value.clone(), DynValue::from(class));
        Ok(())
   } 
}

impl ClassStatement {
    pub fn new(name: Token, methods: Vec<FunctionStatement>) -> Self {
        Self {
            name,
            methods,
        }
    }
}

impl Statement for FunctionStatement {}
impl Statement for ExpressionStatement {}
impl Statement for IfStatement {}
impl Statement for PrintStatement {}
impl Statement for VarStatement {}
impl Statement for BlockStatement {}
impl Statement for WhileStatement {}
impl Statement for ForeachStatement {}
impl Statement for ReturnStatement {}
impl Statement for GlobalStatement {}
impl Statement for NonlocalStatement {}
impl Statement for ClassStatement {}
