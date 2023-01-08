use std::collections::HashMap;
use super::types::DynValue;
use std::rc::Rc;
use std::cell::RefCell;

pub type Env = Rc<RefCell<Environment>>;

pub struct Environment {
    enclosing: Option<Env>,
    values: HashMap<String, DynValue>,
}

impl Environment {
    fn new() -> Self {
        Environment {
            values: HashMap::new(),
            enclosing: None,
        }
    }

    fn new_enclosed(enclosing: Env) -> Self {
        Environment {
            values: HashMap::new(),
            enclosing: Some(enclosing),
        }
    }

    pub fn get(&self, name: &str) -> Option<DynValue> {
        if let Some(value) = self.values.get(name) {
            Some(value.clone())
        } else if let Some(enclosing) = &self.enclosing {
            enclosing.borrow().get(name)
        } else {
            None
        }
    }

    pub fn set(&mut self, name: String, value: DynValue) {
        self.values.insert(name, value);
    }
}

pub fn make_env() -> Env {
    Rc::new(RefCell::new(Environment::new()))
}

pub fn make_env_enclosed(enclosing: Env) -> Env {
    Rc::new(RefCell::new(Environment::new_enclosed(enclosing)))
}
