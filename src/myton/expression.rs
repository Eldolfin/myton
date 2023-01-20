use std::any::Any;

use super::class::Instance;
use super::environment::{Env, make_env_enclosed};
use super::token::{Token, TokenKind};
use super::types::{TypeKind, DynValue};
use super::traceback::Traceback;
use super::resolver::{Resolvable, UUID};

pub trait Evaluable {
    fn eval(&self, env: &Env) -> Result<DynValue, Traceback>;
}

pub trait Expression: Evaluable + Resolvable + Any {
    fn uuid(&self) -> UUID;

    fn as_any(&self) -> &dyn Any;
}

pub type EXPR = Box<dyn Expression>;

pub struct Operator {
    token: Token,
    kind: OperatorKind,
}

pub struct Literal {
    pub token: Token,
    uuid: UUID,
}

pub struct List {
    pub elements: Vec<EXPR>,
    uuid: UUID,
}

#[derive(Clone)]
pub struct Variable {
    pub name: Token,
    uuid: UUID,
}

pub struct Binary {
    pub left: EXPR,
    pub operator: Operator,
    pub right: EXPR,
    uuid: UUID,
}

pub struct Logical {
    pub left: EXPR,
    pub kind: LogicalKind,
    pub right: EXPR,
    uuid: UUID,
}

pub struct Unary {
    pub operator: Operator,
    pub right: EXPR,
    uuid: UUID,
}

pub struct Call {
    pub callee: EXPR,
    pub paren: Token,
    pub arguments: Vec<EXPR>,
    uuid: UUID,
}

pub struct Grouping {
    pub expression: EXPR,
    uuid: UUID,
}

pub struct Get {
    pub object: EXPR,
    pub name: Token,
    uuid: UUID,
}

pub struct Set {
    pub object: EXPR,
    pub name: Token,
    pub value: EXPR,
    uuid: UUID,
}

#[derive(Clone)]
pub struct This {
    pub keyword: Token,
    uuid: UUID,
}

#[derive(Clone)]
pub struct Super {
    pub keyword: Token,
    pub method: Token,
    uuid: UUID,
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
    pub fn new(token: Token, right: EXPR, uuid: UUID) -> Unary {
        let type_ = match token.kind {
            TokenKind::Minus => OperatorKind::Negate,
            TokenKind::Bang => OperatorKind::Not,
            _ => panic!("Invalid token type for unary operator"),
        };

        Unary {
            operator: Operator{token, kind: type_},
            right,
            uuid
        }
    }
}

impl Binary {
    pub fn new(left: EXPR, token: Token, right: EXPR, uuid: UUID) -> Binary {
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
            right,
            uuid,
        }
    }
}

impl Logical {
    pub fn new(left: EXPR, token: Token, right: EXPR, uuid: UUID) -> Logical {
        let kind = match token.kind {
            TokenKind::Or => LogicalKind::Or,
            TokenKind::And => LogicalKind::And,
            _ => panic!("Invalid token type for logical operator"),
        };

        Logical {
            left,
            kind, 
            right,
            uuid,
        }
    }
}

impl Evaluable for Literal {
    fn eval (&self, _: &Env) -> Result<DynValue, Traceback> {
        Ok(DynValue::from_token(&self.token))
    }
}

impl Evaluable for List {
    fn eval(&self, env: &Env) -> Result<DynValue, Traceback> {
        Ok(DynValue::from_vec(self.elements.iter().map(|e| e.eval(env)).collect::<Result<Vec<DynValue>, Traceback>>()?))
    }
}

impl List {
    pub fn new(elements: Vec<EXPR>, uuid: UUID) -> List {
        List {
            elements,
            uuid,
        }
    }
}

impl Literal {
    pub fn new(token: Token, uuid: UUID) -> Literal {
        Literal { token, uuid }
    }
}

impl Evaluable for Variable {
    fn eval (&self, env: &Env) -> Result<DynValue, Traceback> {
        match env.borrow().get_from_variable(self) {
            Some(value) => Ok(value),
            None => Err(Traceback { 
                message: Some(format!("Undefined variable '{}'", self.name.value)),
                pos: self.name.pos.unwrap(),
                ..Default::default()
            })
        }
    }
}

impl Variable {
    pub fn new(token: Token, uuid: UUID) -> Variable {
        Variable { name: token, uuid }
    }
}

impl Evaluable for Grouping {
    fn eval (&self, env: &Env) -> Result<DynValue, Traceback> {
        Ok(self.expression.eval(env)?)
    }
}

impl Grouping {
    pub fn new(expression: EXPR, uuid: UUID) -> Grouping {
        Grouping { expression, uuid }
    }
}

impl Evaluable for Unary {
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

impl Evaluable for Binary {
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
                    TypeKind::List => {let list = left.as_list().unwrap();
                        let num = right.as_number() as usize;
                        Ok(DynValue::from(list.iter().cycle().take(list.len() * num).cloned().collect::<Vec<DynValue>>()))
                    }
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

impl Evaluable for Logical {
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


impl Evaluable for Call {
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
    pub fn new(callee: EXPR, paren: Token, arguments: Vec<EXPR>, uuid: UUID) -> Self {
        Self {
            callee,
            paren,
            arguments,
            uuid,
        }
    }
}

impl Evaluable for Get {
    fn eval(&self, env: &Env) -> Result<DynValue, Traceback> {
        let object = self.object.eval(env)?;
        if let Some(instance) = object.as_instance() {
            if let Some(value) = instance.get(&self.name.value) {
                return Ok(value);
            } else {
                return Err(Traceback {
                    message: Some(format!("'{}' object has no attribute '{}'", instance.class.name, self.name.value)),
                    pos: self.name.pos.unwrap(),
                    ..Default::default()
                });
            }
        } else {
            Err(Traceback {
                message: Some(format!("'{}' object has no attribute '{}'", object.tipe, self.name.value)),
                pos: self.name.pos.unwrap(),
                ..Default::default()
            })
        }
    }
}

impl Get {
    pub fn new(object: EXPR, name: Token, uuid: UUID) -> Self {
        Self {
            object,
            name,
            uuid,
        }
    }
}

impl Evaluable for Set {
    fn eval(&self, env: &Env) -> Result<DynValue, Traceback> {
        let object = self.object.eval(env)?;

        let borrow = object.value.borrow_mut().downcast_mut::<Instance>().cloned();

        if let Some(mut instance) = borrow {
            let value = self.value.eval(env)?;
            instance.set(self.name.value.clone(), value.clone());
            Ok(value)
        } else {
            Err(Traceback {
                message: Some(format!("'{}' object has no attribute '{}'", object.tipe, self.name.value)),
                pos: self.name.pos.unwrap(),
                ..Default::default()
            })
        }
    }
}

impl Set {
    pub fn new(object: EXPR, name: Token, value: EXPR, uuid: UUID) -> Self {
        Self {
            object,
            name,
            value,
            uuid,
        }
    }
}

impl Evaluable for This {
    fn eval(&self, env: &Env) -> Result<DynValue, Traceback> {
        Ok(env.borrow().get("this".to_string()).unwrap().clone())
    }
}

impl This {
    pub fn new(keyword: Token, uuid: UUID) -> Self {
        Self {
            keyword,
            uuid,
        }
    }
}

impl Evaluable for Super {
    fn eval(&self, env: &Env) -> Result<DynValue, Traceback> {
        let var = Variable::new(self.keyword.clone(), self.uuid);

        // unwraps are safe because we check for 
        // errors in the resolver
        let superclass = env.borrow().get_from_variable(&var).unwrap().as_class().unwrap();

        // :(
        let object = env.borrow()
            .enclosing
            .as_ref()
            .unwrap()
            .borrow()
            .get("this".to_string())
            .unwrap()
            .clone()
            .as_instance()
            .unwrap();

        if let Some(method) = superclass.find_method(&self.method.value) {
            Ok(DynValue::from(method.bind(DynValue::from(object))))
        } else {
            Err(Traceback {
                message: Some(format!("Undefined property '{}'", self.method.value)),
                pos: self.method.pos.unwrap(),
                ..Default::default()
            })
        }
    }
}

impl Super {
    pub fn new(keyword: Token, method: Token, uuid: UUID) -> Self {
        Self {
            keyword,
            method,
            uuid,
        }
    }
}

macro_rules! impl_expr {
    ($($t:ty),*) => {
        $(
            impl Expression for $t {
                fn uuid(&self) -> UUID {
                    self.uuid
                }
                
                fn as_any(&self) -> &dyn Any {
                    self
                }
            }
        )*
    }
}
impl_expr!(Unary, Binary, Logical, Call, Grouping, Literal, Variable, List, Get, Set, This, Super);
