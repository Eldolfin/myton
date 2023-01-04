use super::super::token::{Token, TokenKind};

pub trait Expression {
    fn eval(&self) -> i32;
}

pub struct Operator {
    token: Token,
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

pub enum OperatorType {
    Plus,
    Minus,
    Negate,
    Multiply,
    Divide,
    Equal,
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
            TokenKind::Minus => OperatorType::Negate,
            TokenKind::Bang => OperatorType::Not,
            _ => panic!("Invalid token type for unary operator"),
        };

        Unary {
            operator: Operator{token},
            right
        }
    }
}

impl Binary {
    pub fn new(left: Box<dyn Expression>, token: Token, right: Box<dyn Expression>) -> Binary {
        let type_ = match token.kind {
            TokenKind::Plus => OperatorType::Plus,
            TokenKind::Minus => OperatorType::Minus,
            TokenKind::Star => OperatorType::Multiply,
            TokenKind::Slash => OperatorType::Divide,
            TokenKind::EqualEqual => OperatorType::Equal,
            TokenKind::BangEqual => OperatorType::NotEqual,
            TokenKind::Greater => OperatorType::Greater,
            TokenKind::GreaterEqual => OperatorType::GreaterEqual,
            TokenKind::Less => OperatorType::Less,
            TokenKind::LessEqual => OperatorType::LessEqual,
            _ => panic!("Invalid token type for binary operator"),
        };

        Binary {
            left,
            operator: Operator{token},
            right
        }
    }
}

impl Expression for Operator {
    fn eval (&self) -> i32 {
        todo!()
    }
}

impl Expression for Literal {
    fn eval (&self) -> i32 {
        todo!()
    }
}

impl Expression for Binary {
    fn eval (&self) -> i32 {
        todo!()
    }
}

impl Expression for Unary {
    fn eval (&self) -> i32 {
        todo!()
    }
}

impl Expression for Grouping {
    fn eval (&self) -> i32 {
        todo!()
    }
}



pub fn build_ast(tokens: Vec<Token>) -> Box<dyn Expression> {
    todo!()
}
