use crate::myton::MyWrite;

use super::super::environment::{Env, EnvVariable};
use super::super::functions::*;
use super::super::token::{Token, TokenKind};
use super::super::types::{TypeKind, DynValue};
use super::super::traceback::Traceback;
use std::rc::Rc;
use std::cell::RefCell;
use std::io::Write;

pub trait Expression {
    fn eval(&self, env: &Env) -> Result<DynValue, Traceback>;
}

pub trait Statement {
    fn execute(&self, env: &Env) -> Result<(), Traceback>;
}

pub type Expr = Box<dyn Expression>;
pub type Stmt = Box<dyn Statement>;
//
// enum Expressions {
//     Binary(Binary),
//     Unary(Unary),
//     Literal(Literal),
//     Grouping(Grouping),
//     Variable(Variable),
//     Logical(Logical),
//     Call(Call),
// }
//
// enum Statements {
//     Expression(ExpressionStatement),
//     Print(PrintStatement),
//     Var(VarStatement),
//     Block(BlockStatement),
//     If(IfStatement),
//     While(WhileStatement),
//     Function(FunctionStatement),
// }

pub struct Operator {
    token: Token,
    kind: OperatorKind,
}

pub struct Literal {
    pub token: Token,
}

pub struct List {
    pub elements: Vec<Expr>,
}

pub struct Variable {
    pub token: Token,
}

pub struct Binary {
    left: Expr,
    operator: Operator,
    right: Expr,
}

pub struct Logical {
    left: Expr,
    kind: LogicalKind,
    right: Expr,
}

pub struct Unary {
    operator: Operator,
    right: Expr,
}

pub struct Call {
    pub callee: Expr,
    paren: Token,
    arguments: Vec<Expr>,
}

pub struct Grouping {
    pub expression: Expr,
}

pub struct ExpressionStatement {
    pub expression: Expr,
}

pub struct IfStatement {
    pub condition: Expr,
    pub then_branch: Stmt,
    pub else_branch: Option<Stmt>,
}

pub struct WhileStatement {
    pub condition: Expr,
    pub body: Stmt,
}

pub struct ForeachStatement {
    pub variable: String,
    pub collection: Expr,
    pub body: Stmt,
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
    pub statements: Vec<Stmt>,
}

pub struct FunctionStatementInner {
    pub name: String,
    pub parameters: Vec<String>,
    pub body: Stmt,
}

pub struct FunctionStatement {
    pub inner: Rc<RefCell<FunctionStatementInner>>
}

pub struct ReturnStatement {
    pub keyword: Token,
    pub value: Option<Expr>,
}

pub enum OperatorKind {
    Plus,
    Minus,
    Negate,
    Multiply,
    Divide,
    Equal,
    StrictEqual,
    NotEqual,
    Not,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Modulo,
}

pub enum LogicalKind {
    And,
    Or,
}

impl Unary {
    pub fn new(token: Token, right: Expr) -> Unary {
        let type_ = match token.kind {
            TokenKind::Minus => OperatorKind::Negate,
            TokenKind::Bang => OperatorKind::Not,
            _ => panic!("Invalid token type for unary operator"),
        };

        Unary {
            operator: Operator{token, kind: type_},
            right
        }
    }
}

impl Binary {
    pub fn new(left: Expr, token: Token, right: Expr) -> Binary {
        let type_ = match token.kind {
            TokenKind::Plus => OperatorKind::Plus,
            TokenKind::Minus => OperatorKind::Minus,
            TokenKind::Star => OperatorKind::Multiply,
            TokenKind::Slash => OperatorKind::Divide,
            TokenKind::EqualEqual => OperatorKind::Equal,
            TokenKind::EqualEqualEqual => OperatorKind::StrictEqual,
            TokenKind::BangEqual => OperatorKind::NotEqual,
            TokenKind::Greater => OperatorKind::Greater,
            TokenKind::GreaterEqual => OperatorKind::GreaterEqual,
            TokenKind::Less => OperatorKind::Less,
            TokenKind::LessEqual => OperatorKind::LessEqual,
            TokenKind::Percent => OperatorKind::Modulo,
            _ => panic!("Invalid token type for binary operator"),
        };

        Binary {
            left,
            operator: Operator{token, kind: type_},
            right
        }
    }
}

impl Logical {
    pub fn new(left: Expr, token: Token, right: Expr) -> Logical {
        let kind = match token.kind {
            TokenKind::Or => LogicalKind::Or,
            TokenKind::And => LogicalKind::And,
            _ => panic!("Invalid token type for logical operator"),
        };

        Logical {
            left,
            kind, 
            right
        }
    }
}

impl Expression for Literal {
    fn eval (&self, _: &Env) -> Result<DynValue, Traceback> {
        Ok(DynValue::from_token(&self.token))
    }
}

impl Expression for List {
    fn eval(&self, env: &Env) -> Result<DynValue, Traceback> {
        Ok(DynValue::from_vec(self.elements.iter().map(|e| e.eval(env)).collect::<Result<Vec<DynValue>, Traceback>>()?))
    }
}

impl Literal {
    pub fn new(token: Token) -> Literal {
        Literal { token }
    }
}

impl Expression for Variable {
    fn eval (&self, env: &Env) -> Result<DynValue, Traceback> {
        match env.borrow().get(self.token.value.to_string()) {
            Some(value) => Ok(value),
            None => Err(Traceback { 
                message: Some(format!("Undefined variable '{}'", self.token.value)),
                pos: self.token.pos.unwrap(),
                ..Default::default()
            })
        }
    }
}

impl Expression for Grouping {
    fn eval (&self, env: &Env) -> Result<DynValue, Traceback> {
        Ok(self.expression.eval(env)?)
    }
}

impl Expression for Unary {
    fn eval (&self, env: &Env) -> Result<DynValue, Traceback> {
        let right = self.right.eval(env)?;

        match self.operator.kind {
            OperatorKind::Negate => {
                if !right.is_number() {
                    return Err(Traceback { message: Some(format!("bad operand type for unary -: '{}'", right.tipe)), pos: self.operator.token.pos.unwrap(), ..Default::default()});
                }
                Ok(DynValue::from(-right.as_number()))
            },
            OperatorKind::Not => {
                Ok(DynValue::from(!right.as_bool()))
            },
            _ => panic!("Invalid token type for unary operator"),
        }
    }
}

impl Binary {
    fn check_types(&self, left: DynValue, right: DynValue) -> bool {
        match self.operator.kind {
            OperatorKind::Minus | OperatorKind::Divide | OperatorKind::Modulo  => left.is_number() && right.is_number(),
            OperatorKind::Multiply => (!left.is_nil()) && right.is_number(),
            OperatorKind::Greater | OperatorKind::GreaterEqual | OperatorKind::Less | 
                OperatorKind::LessEqual => 
                !left.is_nil() && 
                    (left.tipe == right.tipe || 
                    left.tipe == TypeKind::Number && right.tipe == TypeKind::Boolean || 
                    left.tipe == TypeKind::Boolean && right.tipe == TypeKind::Number),
            OperatorKind::Plus => !(left.is_nil() || right.is_nil()),
            OperatorKind::Equal | OperatorKind::NotEqual | OperatorKind::StrictEqual => true,
            _ => panic!("Invalid token type for binary operator"),
        }
    }
}

impl Expression for Binary {
    fn eval (&self, env: &Env) -> Result<DynValue, Traceback> {
        let left = self.left.eval(env)?;
        let right = self.right.eval(env)?;

        if !self.check_types(left.clone(), right.clone()) {
            return Err(Traceback { message: Some(format!("unsupported operand type(s) for {}: '{}' and '{}'", self.operator.token.value, left.tipe, right.tipe)), pos: self.operator.token.pos.unwrap(), ..Default::default()});
        }

        match self.operator.kind {
            OperatorKind::Plus => {
                if left.is_number() && right.is_number() {
                    Ok(DynValue::from(left.as_number() + right.as_number()))
                } else {
                    Ok(DynValue::from(left.as_string() + &right.as_string()))
                }
            },
            OperatorKind::Minus => {
                Ok(DynValue::from(left.as_number() - right.as_number()))
            },
            OperatorKind::Multiply => {
                match left.tipe {
                    TypeKind::Number => Ok(DynValue::from(left.as_number() * right.as_number())),
                    TypeKind::Stringue => Ok(DynValue::from(left.as_string().repeat(right.as_number() as usize))),
                    _ => panic!("Invalid left type for * operator"),
                }
            },
            OperatorKind::Divide => {
                Ok(DynValue::from(left.as_number() / right.as_number()))
            },
            OperatorKind::Modulo => {
                Ok(DynValue::from(left.as_number() % right.as_number()))
            },
            OperatorKind::Equal => {
                Ok(DynValue::from(left == right))
            },
            OperatorKind::StrictEqual => {
                Ok(DynValue::from(left.tipe == right.tipe && left == right))
            },
            OperatorKind::NotEqual => {
                Ok(DynValue::from(left != right))
            },
            OperatorKind::Greater => {
                Ok(DynValue::from(left > right))
            },
            OperatorKind::GreaterEqual => {
                Ok(DynValue::from(left >= right))
            },
            OperatorKind::Less => {
                Ok(DynValue::from(left < right))
            },
            OperatorKind::LessEqual => {
                Ok(DynValue::from(left <= right))
            },
            _ => panic!("Invalid token type for binary operator"),
        }
    }
}

impl Expression for Logical {
    fn eval(&self, env: &Env) -> Result<DynValue, Traceback> {
        let left = self.left.eval(env)?;

        match self.kind {
            LogicalKind::Or => {
                if left.as_bool() {
                    return Ok(left);
                }
            },
            LogicalKind::And => {
                if !left.as_bool() {
                    return Ok(left);
                }
            },
        }

        self.right.eval(env)
    }
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
    pub fn new(name: String, parameters: Vec<String>, body: Stmt) -> Self {
        Self {
            inner: Rc::new(RefCell::new(FunctionStatementInner {
                name,
                parameters,
                body,
            })),
        }
    }
}

impl Clone for FunctionStatement {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl Expression for Call {
    fn eval(&self, env: &Env) -> Result<DynValue, Traceback> {
        let args = self.arguments.iter().map(|arg| arg.eval(env)).collect::<Result<Vec<_>, _>>()?;
        let maybe_callee = self.callee.eval(env)?;

        if let Some(callee) = maybe_callee.as_callable() {
            if args.len() != callee.arity() {
                return Err(Traceback {
                    message: Some(format!("Expected {} arguments but got {}", callee.arity(), args.len())),
                    pos: self.paren.pos.unwrap(),
                    ..Default::default()
                });
            }
            callee.call(env, args)
        } else {
            Err(Traceback{
                message: Some(format!("'{}' object is not callable", maybe_callee.tipe)),
                pos: self.paren.pos.unwrap(),
                ..Default::default()
            })
        }
    }
}


impl Call {
    pub fn new(callee: Expr, paren: Token, arguments: Vec<Expr>) -> Self {
        Self {
            callee,
            paren,
            arguments,
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
