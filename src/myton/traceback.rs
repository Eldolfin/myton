use super::types::DynValue;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub enum TracebackKind {
    Error,
    // Tracebacks are also a way to return values from functions
    Return,
}

#[derive(Debug, Clone)]
pub struct Traceback {
    pub pos: (usize, usize),
    pub message: Option<String>,
    pub filename: Option<String>,
    pub function_name: Option<String>,
    pub code: Option<String>,
    pub value: Option<DynValue>,
    pub tipe: TracebackKind,
}

impl Default for Traceback {
    fn default() -> Self {
        Self {
            pos: (0, 0),
            message: None,
            filename: None,
            function_name: None,
            code: None,
            value: None,
            tipe: TracebackKind::Error,
        }
    }
}

impl Traceback {
    pub fn from_message(message: &str) -> Self {
        Self {
            message: Some(message.to_string()),
            ..Default::default()
        }
    }

    pub fn from_return_value(value: DynValue) -> Self {
        Self {
            value: Some(value),
            tipe: TracebackKind::Return,
            ..Default::default()
        }
    }
}

impl Display for TracebackKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TracebackKind::Error => write!(f, "error"),
            TracebackKind::Return => write!(f, "return"),
        }
    }
}

impl From<String> for Traceback {
    fn from(message: String) -> Self {
        Self {
            message: Some(message),
            ..Default::default()
        }
    }
}

impl From<&str> for Traceback {
    fn from(message: &str) -> Self {
        Self {
            message: Some(message.to_string()),
            ..Default::default()
        }
    }
}
