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


#[cfg(test)]
mod tests {
    use super::*;
    use super::super::types::{DynValue, TypeKind};
    use super::super::native_functions::native_clock;
    use super::super::functions::NativeFunction;

    #[test]
    fn test_get() {
        let mut env = Environment::new();
        env.set("a".to_string(), DynValue::from(1.0));
        assert!(env.get("a").is_some());
        assert!(env.get("a").unwrap().is_number());
        assert_eq!(env.get("a").unwrap().as_number(), 1.0);
        assert!(env.get("b").is_none());
    }

    #[test]
    fn test_builtin_function() {
        let value = DynValue::new_with_name(Box::new(NativeFunction{ nb_args: 0, func: native_clock }), TypeKind::NativeFunction, "clock".to_string());
        let env = make_env();
        env.borrow_mut().set("clock".to_string(), value);

        let value = env.borrow().get("clock");

        assert!(value.is_some());

        let value = value.unwrap();

        assert_eq!(value.tipe, TypeKind::NativeFunction);

        assert_eq!(value.name, Some("clock".to_string()));

        assert!(value.as_callable().is_some());

        assert!(value.as_callable().unwrap().call(&env, vec![]).unwrap().as_number() > 1673047730.0);
    }

    #[test]
    fn test_enclosing() {
        let global = make_env();
        let local = make_env_enclosed(global.clone());
        let mut borrowed_local = local.borrow_mut();

        global.borrow_mut().set("a".to_string(), DynValue::from(1.0));
        global.borrow_mut().set("c".to_string(), DynValue::from(2.0));

        borrowed_local.set("b".to_string(), DynValue::from(2.0));
        borrowed_local.set("a".to_string(), DynValue::from(3.0));

        assert!(borrowed_local.get("a").is_some());
        assert!(borrowed_local.get("a").unwrap().is_number());
        assert_eq!(borrowed_local.get("a").unwrap().as_number(), 3.0);

        assert!(borrowed_local.get("b").is_some());
        assert!(borrowed_local.get("b").unwrap().is_number());
        assert_eq!(borrowed_local.get("b").unwrap().as_number(), 2.0);

        assert!(borrowed_local.get("c").is_some());
        assert!(borrowed_local.get("c").unwrap().is_number());
        assert_eq!(borrowed_local.get("c").unwrap().as_number(), 2.0);

        assert!(borrowed_local.get("d").is_none());

        let mut borrowed_global = global.borrow_mut();
        assert!(borrowed_global.get("a").is_some());
        assert!(borrowed_global.get("a").unwrap().is_number());
        assert_eq!(borrowed_global.get("a").unwrap().as_number(), 1.0);
    }
}
