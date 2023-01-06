use crate::myton::environment::Environment;

use super::super::token::{Token, TokenKind};
use super::super::types::{TypeKind, DynValue};
use super::super::traceback::Traceback;

pub trait Expression {
    fn eval(&self, env: &mut Environment) -> Result<DynValue, Traceback>;
}

pub trait Statement {
    fn execute(&mut self, env: &mut Environment) -> Result<String, Traceback>;
}

pub struct Operator {
    token: Token,
    kind: OperatorKind,
}

pub struct Literal {
    pub token: Token,
}

pub struct List {
    pub elements: Vec<Box<dyn Expression>>,
}

pub struct Variable {
    pub token: Token,
}

pub struct Binary {
    left: Box<dyn Expression>,
    operator: Operator,
    right: Box<dyn Expression>,
}

pub struct Logical {
    left: Box<dyn Expression>,
    kind: LogicalKind,
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

pub struct IfStatement {
    pub condition: Box<dyn Expression>,
    pub then_branch: Box<dyn Statement>,
    pub else_branch: Option<Box<dyn Statement>>,
}

pub struct WhileStatement {
    pub condition: Box<dyn Expression>,
    pub body: Box<dyn Statement>,
}

pub struct ForeachStatement {
    pub variable: String,
    pub collection: Box<dyn Expression>,
    pub body: Box<dyn Statement>,
}


pub struct PrintStatement {
    pub expression: Box<dyn Expression>,
}

pub struct VarStatement {
    pub name: String,
    pub initializer: Option<Box<dyn Expression>>,
}

pub struct BlockStatement {
    pub statements: Vec<Box<dyn Statement>>,
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
    pub fn new(left: Box<dyn Expression>, token: Token, right: Box<dyn Expression>) -> Logical {
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
    fn eval (&self, _: &mut Environment) -> Result<DynValue, Traceback> {
        Ok(DynValue::from_token(&self.token))
    }
}

impl Expression for List {
    fn eval(&self, env: &mut Environment) -> Result<DynValue, Traceback> {
        Ok(DynValue::from_vec(self.elements.iter().map(|e| e.eval(env)).collect::<Result<Vec<DynValue>, Traceback>>()?))
    }
}

impl Literal {
    pub fn new(token: Token) -> Literal {
        Literal { token }
    }
}

impl Expression for Variable {
    fn eval (&self, env: &mut Environment) -> Result<DynValue, Traceback> {
        match env.get(&self.token.value) {
            Some(value) => Ok(value),
            None => Err(Traceback{ message: Some(format!("Undefined variable '{}'", self.token.value)), ..Default::default()})
        }
    }
}

impl Expression for Grouping {
    fn eval (&self, env: &mut Environment) -> Result<DynValue, Traceback> {
        Ok(self.expression.eval(env)?)
    }
}

impl Expression for Unary {
    fn eval (&self, env: &mut Environment) -> Result<DynValue, Traceback> {
        let right = self.right.eval(env)?;

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
    fn eval (&self, env: &mut Environment) -> Result<DynValue, Traceback> {
        let left = self.left.eval(env)?;
        let right = self.right.eval(env)?;

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
            OperatorKind::Modulo => {
                Ok(DynValue::new(Box::new(left.as_number() % right.as_number()), TypeKind::Number))
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
                Ok(DynValue::new(Box::new(left > right), TypeKind::Boolean))
            },
            OperatorKind::GreaterEqual => {
                Ok(DynValue::new(Box::new(left >= right), TypeKind::Boolean))
            },
            OperatorKind::Less => {
                Ok(DynValue::new(Box::new(left < right), TypeKind::Boolean))
            },
            OperatorKind::LessEqual => {
                Ok(DynValue::new(Box::new(left <= right), TypeKind::Boolean))
            },
            _ => panic!("Invalid token type for binary operator"),
        }
    }
}

impl Expression for Logical {
    fn eval(&self, env: &mut Environment) -> Result<DynValue, Traceback> {
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
    fn execute(&mut self, env: &mut Environment) -> Result<String, Traceback> {
        self.expression.eval(env)?;
        Ok(String::new())
    }
}

impl Statement for IfStatement {
    fn execute(&mut self, env: &mut Environment) -> Result<String, Traceback> {
        if self.condition.eval(env)?.as_bool() {
            self.then_branch.execute(env)
        } else if let Some(else_branch) = &mut self.else_branch {
            else_branch.execute(env)
        } else {
            Ok(String::new())
        }
    }
}

impl Statement for PrintStatement {
    fn execute(&mut self, env: &mut Environment) -> Result<String, Traceback> {
        let value = self.expression.eval(env)?;
        Ok(format!("{}\n", value.as_string()))
    }
}

impl Statement for VarStatement {
    fn execute(&mut self, env: &mut Environment) -> Result<String, Traceback> {
        let value = match &self.initializer {
            Some(expr) => expr.eval(env)?,
            None => DynValue::new(Box::new(()), TypeKind::Nil),
        };

        env.set(self.name.clone(), value);

        Ok(String::new())
    }
}

impl Statement for BlockStatement {
    fn execute(&mut self, env: &mut Environment) -> Result<String, Traceback> {
        let mut result = String::new();
        for statement in &mut self.statements {
            result += &statement.execute(env)?;
        }
        Ok(result)
    }
}

impl Statement for WhileStatement {
    fn execute(&mut self, env: &mut Environment) -> Result<String, Traceback> {
        let mut result = String::new();

        while self.condition.eval(env)?.as_bool() {
            result += &self.body.execute(env)?;
        }
        Ok(result)
    }
}

impl Statement for ForeachStatement {
    fn execute(&mut self, env: &mut Environment) -> Result<String, Traceback> {
        let mut result = String::new();
        let list = self.collection.eval(env)?;
        if let Some(array) = list.as_list() {

            for value in array {
                env.set(self.variable.clone(), value);
                result += &self.body.execute(env)?;
            }
            Ok(result)
        } else {
            Err(Traceback {
                message: Some(format!("'{}' object is not iterable", list.tipe)),
                ..Default::default()})
        }
    }
}
