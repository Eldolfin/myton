use std::collections::HashMap;
use super::types::DynValue;

pub struct Environment {
    enclosing: Option<Box<Environment>>,
    values: HashMap<String, DynValue>,
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            values: HashMap::new(),
            enclosing: None,
        }
    }

    pub fn new_enclosed(enclosing: Environment) -> Environment {
        Environment {
            values: HashMap::new(),
            enclosing: Some(Box::new(enclosing)),
        }
    }

    pub fn get(&mut self, name: &str) -> Option<DynValue> {
        if let Some(value) = self.values.get(name) {
            Some(value.clone())
        } else if let Some(enclosing) = &mut self.enclosing {
            enclosing.get(name)
        } else {
            None
        }
    }

    pub fn set(&mut self, name: String, value: DynValue) {
        self.values.insert(name, value);
    }
}
