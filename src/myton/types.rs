use super::class::{Class, Instance};
use super::functions::{Callable, Function, NativeFunction};
use super::token::{Token, TokenKind};
use std::any::Any;
use std::cell::RefCell;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeKind {
    Number,
    Stringue,
    Boolean,
    Nil,
    List,
    Function,
    NativeFunction,
    Class,
    Instance,
}

#[derive(Debug)]
pub struct DynValue {
    pub value: Rc<RefCell<Box<dyn Any>>>,
    pub name: Option<String>,
    pub tipe: TypeKind,
}

impl TypeKind {
    fn from_token(token: &Token) -> Self {
        match token.kind {
            TokenKind::Number => Self::Number,
            TokenKind::Stringue => Self::Stringue,
            TokenKind::True => Self::Boolean,
            TokenKind::False => Self::Boolean,
            TokenKind::Nil => Self::Nil,
            _ => panic!("Invalid token type for literal"),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::Number => "number".to_string(),
            Self::Stringue => "str".to_string(),
            Self::Boolean => "bool".to_string(),
            Self::Nil => "NoneType".to_string(),
            Self::List => "list".to_string(),
            Self::Function => "function".to_string(),
            Self::NativeFunction => "built-in function".to_string(),
            Self::Class => "class".to_string(),
            Self::Instance => "object".to_string(),
        }
    }
}

impl Display for TypeKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.to_string())
    }
}

impl PartialEq for DynValue {
    fn eq(&self, other: &Self) -> bool {
        let a = if self.tipe == TypeKind::Boolean {
            self.as_number().to_string()
        } else if self.tipe == TypeKind::Number && self.as_number().is_nan() {
            ("nan".to_owned() + &self.as_number().to_string()).to_string()
        } else {
            self.as_string()
        };

        let b = if other.tipe == TypeKind::Boolean {
            other.as_number().to_string()
        } else {
            other.as_string()
        };

        a == b
    }
}

impl PartialOrd for DynValue {
    fn partial_cmp(&self, other: &DynValue) -> Option<std::cmp::Ordering> {
        match (self.tipe.clone(), other.tipe.clone()) {
            (TypeKind::Number, TypeKind::Number)
            | (TypeKind::Number, TypeKind::Boolean)
            | (TypeKind::Boolean, TypeKind::Number)
            | (TypeKind::Boolean, TypeKind::Boolean) => {
                let a = self.as_number();
                let b = other.as_number();
                a.partial_cmp(&b)
            }
            (TypeKind::Stringue, TypeKind::Stringue) => {
                let a = self.as_string();
                let b = other.as_string();
                a.partial_cmp(&b)
            }
            (TypeKind::List, TypeKind::List) => {
                let a = self.as_list();
                let b = other.as_list();
                a.partial_cmp(&b)
            }
            _ => None,
        }
    }
}

impl Clone for DynValue {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            tipe: self.tipe.clone(),
            name: self.name.clone(),
        }
    }
}

impl Display for DynValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "[{}('{}')]", self.tipe, self.as_string())
    }
}

impl DynValue {
    pub fn new(value: Box<dyn Any>, tipe: TypeKind) -> Self {
        Self {
            value: Rc::new(RefCell::new(value)),
            tipe,
            name: None,
        }
    }

    pub fn new_with_name(value: Box<dyn Any>, tipe: TypeKind, name: String) -> Self {
        Self {
            value: Rc::new(RefCell::new(value)),
            tipe,
            name: Some(name),
        }
    }

    pub fn from_token(token: &Token) -> Self {
        match TypeKind::from_token(token) {
            TypeKind::Number => Self::from_f64(token.value.parse::<f64>().unwrap()),
            TypeKind::Stringue => Self::from_string(token.value.clone()),
            TypeKind::Boolean => Self::from_bool(token.kind == TokenKind::True),
            TypeKind::Nil => Self::none(),
            _ => panic!("Invalid token type for literal"),
        }
    }

    pub fn from_f64(value: f64) -> Self {
        Self::new(Box::new(value), TypeKind::Number)
    }

    pub fn from_string(value: String) -> Self {
        Self::new(Box::new(value), TypeKind::Stringue)
    }

    pub fn from_bool(value: bool) -> Self {
        Self::new(Box::new(value), TypeKind::Boolean)
    }

    pub fn from_vec(value: Vec<DynValue>) -> Self {
        Self::new(Box::new(value), TypeKind::List)
    }

    pub fn from_function(value: Function, name: String) -> Self {
        Self::new_with_name(Box::new(value), TypeKind::Function, name)
    }

    pub fn from_native_function(value: NativeFunction, name: String) -> Self {
        Self::new_with_name(Box::new(value), TypeKind::NativeFunction, name)
    }

    pub fn none() -> Self {
        Self::new(Box::new(()), TypeKind::Nil)
    }

    pub fn as_number(&self) -> f64 {
        match self.tipe {
            TypeKind::Number => *self.value.borrow().downcast_ref::<f64>().unwrap(),
            TypeKind::Stringue => self.as_string().parse::<f64>().unwrap(),
            TypeKind::Boolean => {
                if self.as_bool() {
                    1.0
                } else {
                    0.0
                }
            }
            TypeKind::Nil => 0.0,
            _ => panic!("Invalid type for number"),
        }
    }

    pub fn as_string(&self) -> String {
        match self.tipe {
            TypeKind::Number => self.as_number().to_string(),
            TypeKind::Stringue => self
                .value
                .borrow()
                .downcast_ref::<String>()
                .unwrap()
                .clone(),
            TypeKind::Boolean => if self.as_bool() { "True" } else { "False" }.to_string(),
            TypeKind::Nil => "None".to_string(),
            TypeKind::List => format!(
                "[{}]",
                &self
                    .as_list()
                    .unwrap()
                    .iter()
                    .map(|x| x.as_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            TypeKind::Instance => format!(
                "<{} object>",
                self.as_instance().unwrap().borrow().class.name
            ),
            _ => format!(
                "<{} {}>",
                self.tipe,
                self.name.as_ref().unwrap_or(&"unnamed".to_string())
            ),
        }
    }

    pub fn as_bool(&self) -> bool {
        match self.tipe {
            TypeKind::Number => self.as_number() != 0.0,
            TypeKind::Stringue => !self.as_string().is_empty(),
            TypeKind::Boolean => *self.value.borrow().downcast_ref::<bool>().unwrap(),
            TypeKind::Nil => false,
            TypeKind::List => !self.as_list().unwrap().is_empty(),
            TypeKind::Function
            | TypeKind::NativeFunction
            | TypeKind::Class
            | TypeKind::Instance => true,
        }
    }

    pub fn as_list(&self) -> Option<Vec<DynValue>> {
        if self.tipe == TypeKind::List {
            Some(
                self.value
                    .borrow()
                    .downcast_ref::<Vec<DynValue>>()
                    .unwrap()
                    .clone(),
            )
        } else {
            None
        }
    }

    pub fn as_callable(&self) -> Option<Box<dyn Callable>> {
        match self.tipe {
            TypeKind::Function => Some(Box::new(
                self.value
                    .borrow()
                    .downcast_ref::<Function>()
                    .unwrap()
                    .clone(),
            )),
            TypeKind::NativeFunction => Some(Box::new(
                self.value
                    .borrow()
                    .downcast_ref::<NativeFunction>()
                    .unwrap()
                    .clone(),
            )),
            TypeKind::Class => Some(Box::new(
                self.value.borrow().downcast_ref::<Class>().unwrap().clone(),
            )),
            _ => None,
        }
    }

    pub fn as_instance(&self) -> Option<Rc<RefCell<Instance>>> {
        if self.tipe == TypeKind::Instance {
            Some(
                self.value
                    .borrow()
                    .downcast_ref::<Rc<RefCell<Instance>>>()
                    .unwrap()
                    .clone(),
            )
        } else {
            None
        }
    }

    pub fn as_class(&self) -> Option<Class> {
        if self.tipe == TypeKind::Class {
            Some(self.value.borrow().downcast_ref::<Class>().unwrap().clone())
        } else {
            None
        }
    }

    pub fn is_nil(&self) -> bool {
        self.tipe == TypeKind::Nil
    }

    pub fn is_number(&self) -> bool {
        self.tipe == TypeKind::Number
            || self.tipe == TypeKind::Boolean
            || (self.tipe == TypeKind::Stringue && self.as_string().parse::<f64>().is_ok())
    }

    pub fn increment_by(&mut self, incrementation_value: f64) {
        if self.tipe == TypeKind::Number {
            let mut value = self.value.borrow_mut();
            let value = value.downcast_mut::<f64>().unwrap();
            *value += incrementation_value;
        } else {
            panic!("Invalid type for increment {}", self.tipe);
        }
    }
}

impl From<f64> for DynValue {
    fn from(value: f64) -> Self {
        Self::from_f64(value)
    }
}

impl From<String> for DynValue {
    fn from(value: String) -> Self {
        Self::from_string(value)
    }
}

impl From<bool> for DynValue {
    fn from(value: bool) -> Self {
        Self::from_bool(value)
    }
}

impl From<Vec<DynValue>> for DynValue {
    fn from(value: Vec<DynValue>) -> Self {
        Self::from_vec(value)
    }
}

impl From<Token> for DynValue {
    fn from(token: Token) -> Self {
        Self::from_token(&token)
    }
}

impl From<i32> for DynValue {
    fn from(value: i32) -> Self {
        Self::from_f64(value as f64)
    }
}

impl From<Class> for DynValue {
    fn from(value: Class) -> Self {
        let name = value.name.clone();
        Self::new_with_name(Box::new(value), TypeKind::Class, name)
    }
}

impl From<Instance> for DynValue {
    fn from(instance: Instance) -> Self {
        Self::new(Box::new(instance), TypeKind::Instance)
    }
}

impl From<Rc<RefCell<Instance>>> for DynValue {
    fn from(instance: Rc<RefCell<Instance>>) -> Self {
        Self::new(Box::new(instance), TypeKind::Instance)
    }
}

impl From<Function> for DynValue {
    fn from(value: Function) -> Self {
        let name = value.statement.inner.borrow().name.value.clone();
        Self::from_function(value, name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number() {
        let value = DynValue::from(1.0);
        assert_eq!(value.as_number(), 1.0);
        assert_eq!(value.as_string(), "1");
        assert_eq!(value.as_bool(), true);
        assert_eq!(value.is_nil(), false);
        assert_eq!(value.is_number(), true);
    }

    #[test]
    fn test_string() {
        let value = DynValue::from("Hello".to_string());
        // assert_eq!(value.as_number(), 0.0);
        assert_eq!(value.as_string(), "Hello");
        assert_eq!(value.as_bool(), true);
        assert_eq!(value.is_nil(), false);
        assert_eq!(value.is_number(), false);
    }

    #[test]
    fn test_boolean() {
        let value = DynValue::from(true);
        assert_eq!(value.as_number(), 1.0);
        assert_eq!(value.as_string(), "True");
        assert_eq!(value.as_bool(), true);
        assert_eq!(value.is_nil(), false);
        assert_eq!(value.is_number(), true);
    }

    #[test]
    fn test_nil() {
        let value = DynValue::none();
        assert_eq!(value.as_number(), 0.0);
        assert_eq!(value.as_string(), "None");
        assert_eq!(value.as_bool(), false);
        assert_eq!(value.is_nil(), true);
        assert_eq!(value.is_number(), false);
    }

    #[test]
    fn test_from_token() {
        let token = Token {
            kind: TokenKind::Number,
            value: "0".to_string(),
            ..Default::default()
        };
        let value = DynValue::from(token);
        assert_eq!(value.as_number(), 0.0);
        assert_eq!(value.as_string(), "0");
        assert_eq!(value.as_bool(), false);
        assert_eq!(value.is_nil(), false);
        assert_eq!(value.is_number(), true);
    }

    #[test]
    fn test_list() {
        let value = DynValue::from(vec![DynValue::from_f64(1.0), DynValue::from_f64(2.0)]);
        assert_eq!(value.as_string(), "[1, 2]");
        assert_eq!(value.as_bool(), true);
        assert_eq!(value.is_nil(), false);
        assert_eq!(value.is_number(), false);
    }

    #[test]
    fn test_function() {
        let value = DynValue::new_with_name(
            Box::new(|_: Vec<DynValue>| DynValue::none()),
            TypeKind::Function,
            "test".to_string(),
        );
        assert_eq!(value.as_string(), "<function test>");
        assert_eq!(value.as_bool(), true);
        assert_eq!(value.is_nil(), false);
        assert_eq!(value.is_number(), false);
    }
}
