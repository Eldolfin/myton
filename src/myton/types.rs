use super::token::{Token, TokenKind};
use std::any::Any;
use std::fmt::{Formatter, Display, Result as FmtResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeKind {
    Number,
    Stringue,
    Boolean,
    Nil,
    List,
}

#[derive(Debug)]
pub struct DynValue {
    pub value: Box<dyn Any>,
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
            (TypeKind::Number, TypeKind::Number) | 
            (TypeKind::Number, TypeKind::Boolean)|
            (TypeKind::Boolean, TypeKind::Number)|
            (TypeKind::Boolean, TypeKind::Boolean) => {
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
        let value : Box<dyn Any> = match self.tipe {
            TypeKind::Number => Box::new(self.as_number()),
            TypeKind::Stringue => Box::new(self.as_string()),
            TypeKind::Boolean => Box::new(self.as_bool()),
            TypeKind::Nil => Box::new(self.is_nil()),
            TypeKind::List => Box::new(self.as_list()),
        };
        Self {
            value,
            tipe: self.tipe.clone(),
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
        Self { value, tipe }
    }

    pub fn from_vec(vec: Vec<DynValue>) -> Self {
        Self {
            value: Box::new(vec),
            tipe: TypeKind::List,
        }
    }

    pub fn from_token(token: &Token) -> Self {
        let type_ = TypeKind::from_token(token);
        let value: Box<dyn Any> = match type_ {
            TypeKind::Number => Box::new(token.value.parse::<f64>().unwrap()),
            TypeKind::Stringue => Box::new(token.value.clone()),
            TypeKind::Boolean => Box::new(token.kind == TokenKind::True),
            TypeKind::Nil => Box::new(()),
            _ => panic!("Invalid token type for literal"),
        };

        Self::new(value, type_)
    }

    pub fn as_number(&self) -> f64 {
        match self.tipe {
            TypeKind::Number => *self.value.downcast_ref::<f64>().unwrap(),
            TypeKind::Stringue => self.as_string().parse::<f64>().unwrap(),
            TypeKind::Boolean => if self.as_bool() {1.0} else {0.0}
            TypeKind::Nil => 0.0,
            _ => panic!("Invalid type for number"),
        }
    }

    pub fn as_string(&self) -> String {
        match self.tipe {
            TypeKind::Number => self.as_number().to_string(),
            TypeKind::Stringue => self.value.downcast_ref::<String>().unwrap().clone(),
            TypeKind::Boolean => if self.as_bool() {"True"} else {"False"}.to_string(),
            TypeKind::Nil => "None".to_string(),
            TypeKind::List =>
                format!("[{}]", &self.as_list().unwrap().iter().map(|x| x.as_string()).collect::<Vec<String>>().join(", ")),
        }
    }

    pub fn as_bool(&self) -> bool {
        match self.tipe {
            TypeKind::Number => self.as_number() != 0.0,
            TypeKind::Stringue => !self.as_string().is_empty(),
            TypeKind::Boolean => *self.value.downcast_ref::<bool>().unwrap(),
            TypeKind::Nil => false,
            TypeKind::List => !self.as_list().unwrap().is_empty(),
        }
    }

    pub fn as_list(&self) -> Option<Vec<DynValue>> {
        if self.tipe == TypeKind::List {
            Some(self.value.downcast_ref::<Vec<DynValue>>().unwrap().clone())
        } else {
            None
        }
    }

    pub fn is_nil(&self) -> bool {
        self.tipe == TypeKind::Nil
    }

    pub fn is_number(&self) -> bool {
        self.tipe == TypeKind::Number || self.tipe == TypeKind::Boolean || 
        (self.tipe == TypeKind::Stringue && self.as_string().parse::<f64>().is_ok())
    }

    pub fn is_string(&self) -> bool {
        self.tipe == TypeKind::Stringue
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number() {
        let value = DynValue::new(Box::new(1.0), TypeKind::Number);
        assert_eq!(value.as_number(), 1.0);
        assert_eq!(value.as_string(), "1");
        assert_eq!(value.as_bool(), true);
        assert_eq!(value.is_nil(), false);
        assert_eq!(value.is_number(), true);
        assert_eq!(value.is_string(), false);
    }

    #[test]
    fn test_string() {
        let value = DynValue::new(Box::new("Hello".to_string()), TypeKind::Stringue);
        // assert_eq!(value.as_number(), 0.0);
        assert_eq!(value.as_string(), "Hello");
        assert_eq!(value.as_bool(), true);
        assert_eq!(value.is_nil(), false);
        assert_eq!(value.is_number(), false);
        assert_eq!(value.is_string(), true);
    }

    #[test]
    fn test_boolean() {
        let value = DynValue::new(Box::new(true), TypeKind::Boolean);
        assert_eq!(value.as_number(), 1.0);
        assert_eq!(value.as_string(), "True");
        assert_eq!(value.as_bool(), true);
        assert_eq!(value.is_nil(), false);
        assert_eq!(value.is_number(), true);
        assert_eq!(value.is_string(), false);
    }

    #[test]
    fn test_nil() {
        let value = DynValue::new(Box::new(()), TypeKind::Nil);
        assert_eq!(value.as_number(), 0.0);
        assert_eq!(value.as_string(), "None");
        assert_eq!(value.as_bool(), false);
        assert_eq!(value.is_nil(), true);
        assert_eq!(value.is_number(), false);
        assert_eq!(value.is_string(), false);
    }

    #[test]
    fn test_from_token() {
        let token = Token { kind: TokenKind::Number, value: "0".to_string() , ..Default::default() };
        let value = DynValue::from_token(&token);
        assert_eq!(value.as_number(), 0.0);
        assert_eq!(value.as_string(), "0");
        assert_eq!(value.as_bool(), false);
        assert_eq!(value.is_nil(), false);
        assert_eq!(value.is_number(), true);
        assert_eq!(value.is_string(), false);
    }
}
