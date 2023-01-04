use super::super::token::{Token, TokenKind};
use super::super::types::{TypeKind, DynValue};
use super::super::traceback::Traceback;

pub trait Expression {
    fn eval(&self) -> Result<DynValue, Traceback>;
}

pub trait Statement {
    fn execute(&self) -> Result<(), Traceback>;
}

pub struct Operator {
    token: Token,
    kind: OperatorKind,
}

pub struct Literal {
    pub token: Token,
}

pub struct Binary {
    left: Box<dyn Expression>,
    operator: Operator,
    right: Box<dyn Expression>,
}

pub struct Unary {
    operator: Operator,
    right: Box<dyn Expression>,
}

pub struct Grouping {
    pub expression: Box<dyn Expression>,
}

pub struct ExpressionStatement {
    pub expression: Box<dyn Expression>,
}

pub struct PrintStatement {
    pub expression: Box<dyn Expression>,
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
}

impl Unary {
    pub fn new(token: Token, right: Box<dyn Expression>) -> Unary {
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
    pub fn new(left: Box<dyn Expression>, token: Token, right: Box<dyn Expression>) -> Binary {
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
            _ => panic!("Invalid token type for binary operator"),
        };

        Binary {
            left,
            operator: Operator{token, kind: type_},
            right
        }
    }
}

impl Expression for Literal {
    fn eval (&self) -> Result<DynValue, Traceback> {
        Ok(DynValue::from_token(&self.token))
    }
}

impl Expression for Grouping {
    fn eval (&self) -> Result<DynValue, Traceback> {
        Ok(self.expression.eval()?)
    }
}

impl Expression for Unary {
    fn eval (&self) -> Result<DynValue, Traceback> {
        let right = self.right.eval()?;

        match self.operator.kind {
            OperatorKind::Negate => {
                if !right.is_number() {
                    return Err(Traceback { message: Some(format!("bad operand type for unary -: '{}'", right.tipe)), pos: self.operator.token.pos.unwrap(), ..Default::default()});
                }
                Ok(DynValue::new(Box::new(-right.as_number()), TypeKind::Number))
            },
            OperatorKind::Not => {
                Ok(DynValue::new(Box::new(!right.as_bool()), TypeKind::Boolean))
            },
            _ => panic!("Invalid token type for unary operator"),
        }
    }
}

impl Binary {
    fn check_types(&self, left: DynValue, right: DynValue) -> bool {
        match self.operator.kind {
            OperatorKind::Minus | OperatorKind::Divide => left.is_number() && right.is_number(),
            OperatorKind::Multiply => (!left.is_nil()) && right.is_number(),
            OperatorKind::Greater | OperatorKind::GreaterEqual | OperatorKind::Less | 
                OperatorKind::LessEqual | OperatorKind::Plus => !(left.is_nil() || right.is_nil()),
            OperatorKind::Equal | OperatorKind::NotEqual | OperatorKind::StrictEqual => true,
            _ => panic!("Invalid token type for binary operator"),
        }
    }
}

impl Expression for Binary {
    fn eval (&self) -> Result<DynValue, Traceback> {
        let left = self.left.eval()?;
        let right = self.right.eval()?;

        if !self.check_types(left.clone(), right.clone()) {
            return Err(Traceback { message: Some(format!("unsupported operand type(s) for {}: '{}' and '{}'", self.operator.token.value, left.tipe, right.tipe)), pos: self.operator.token.pos.unwrap(), ..Default::default()});
        }

        match self.operator.kind {
            OperatorKind::Plus => {
                if left.is_number() && right.is_number() {
                    Ok(DynValue::new(Box::new(left.as_number() + right.as_number()), TypeKind::Number))
                } else {
                    Ok(DynValue::new(Box::new(left.as_string() + &right.as_string()), TypeKind::Stringue))
                }
            },
            OperatorKind::Minus => {
                Ok(DynValue::new(Box::new(left.as_number() - right.as_number()), TypeKind::Number))
            },
            OperatorKind::Multiply => {
                match left.tipe {
                    TypeKind::Number => Ok(DynValue::new(Box::new(left.as_number() * right.as_number()), TypeKind::Number)),
                    TypeKind::Stringue => Ok(DynValue::new(Box::new(left.as_string().repeat(right.as_number() as usize)), TypeKind::Stringue)),
                    _ => panic!("Invalid left type for * operator"),
                }
            },
            OperatorKind::Divide => {
                Ok(DynValue::new(Box::new(left.as_number() / right.as_number()), TypeKind::Number))
            },
            OperatorKind::Equal => {
                Ok(DynValue::new(Box::new(left == right), TypeKind::Boolean))
            },
            OperatorKind::StrictEqual => {
                Ok(DynValue::new(Box::new(left.tipe == right.tipe && left == right), TypeKind::Boolean))
            },
            OperatorKind::NotEqual => {
                Ok(DynValue::new(Box::new(left != right), TypeKind::Boolean))
            },
            OperatorKind::Greater => {
                Ok(DynValue::new(Box::new(left.as_number() > right.as_number()), TypeKind::Boolean))
            },
            OperatorKind::GreaterEqual => {
                Ok(DynValue::new(Box::new(left.as_number() >= right.as_number()), TypeKind::Boolean))
            },
            OperatorKind::Less => {
                Ok(DynValue::new(Box::new(left.as_number() < right.as_number()), TypeKind::Boolean))
            },
            OperatorKind::LessEqual => {
                Ok(DynValue::new(Box::new(left.as_number() <= right.as_number()), TypeKind::Boolean))
            },
            _ => panic!("Invalid token type for binary operator"),
        }
    }
}

impl Statement for ExpressionStatement {
    fn execute(&self) -> Result<(), Traceback> {
        self.expression.eval()?;
        Ok(())
    }
}

impl Statement for PrintStatement {
    fn execute(&self) -> Result<(), Traceback> {
        let value = self.expression.eval()?;
        println!("{}", value);
        Ok(())
    }
}
