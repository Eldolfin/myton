use super::environment::Env;
use super::functions::*;
use super::token::{Token, TokenKind};
use super::types::{TypeKind, DynValue};
use super::traceback::Traceback;

pub trait Expression {
    fn eval(&self, env: &Env) -> Result<DynValue, Traceback>;
}

pub type Expr = Box<dyn Expression>;

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
