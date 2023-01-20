use std::collections::HashMap;

use super::expression::{Variable, Expression};
use super::resolver::UUID;
use super::types::DynValue;
use std::rc::Rc;
use std::cell::RefCell;

pub type Env = Rc<RefCell<Environment>>;

pub struct Environment {
    values: HashMap<String, DynValue>,
    pub enclosing: Option<Env>,
    resolved_locals: Option<HashMap<UUID, usize>>,
    globals :Vec<String>,
    non_locals :Vec<String>,
}

impl Environment {
    fn new() -> Self {
        Environment {
            values: HashMap::new(),
            enclosing: None,
            resolved_locals: None,
            globals: Vec::new(),
            non_locals: Vec::new(),
        }
    }

    fn new_enclosed(enclosing: Env) -> Self {
        Environment {
            values: HashMap::new(),
            enclosing: Some(enclosing.clone()),
            resolved_locals: enclosing.borrow().resolved_locals.clone(),
            globals: enclosing.borrow().globals.clone(),
            non_locals: enclosing.borrow().non_locals.clone(),
        }
    }

    pub fn get(&self, name: String) -> Option<DynValue> {
        if let Some(value) = self.values.get(&name) {
            Some(value.clone())
        } else if let Some(enclosing) = &self.enclosing {
            enclosing.borrow_mut().get(name)
        } else {
            None
        }
    }
    
    // tries to get the value from the resolved
    // locals, if it fails, it tries to get it
    // with the name, with get
    pub fn get_from_variable(&self, variable: &Variable) -> Option<DynValue> {
        if let Some(locals) = &self.resolved_locals {
            if let Some(distance) = locals.get(&variable.uuid()) {
                if let Some(enclosing) = self.ancestor(*distance) {
                    return enclosing.borrow_mut().get(variable.name.value.to_string());
                }
            }
        }
        self.get(variable.name.value.to_string())
    }

    pub fn set(&mut self, name: String, value: DynValue) {
        if self.globals.contains(&name) {
            self.set_global_variable(name, value);
        } else if self.non_locals.contains(&name) {
            if let Some(enclosing) = &self.enclosing {
                enclosing.borrow_mut().set(name, value);
            }
        } else {
            self.values.insert(name, value);
        }
    }

    fn set_global_variable(&mut self, name: String, value: DynValue) {
        if let Some(enclosing) = &self.enclosing {
            enclosing.borrow_mut().set_global_variable(name, value);
        } else {
            self.set(name, value);
        }
    }

    pub fn get_env_var(&self, var: EnvVariable) -> DynValue {
        let name = var.get_name();
        
        self.get(name.to_string()).unwrap_or_else(|| {
            panic!("Undefined environment variable variable '{}'", name.to_string());
        })
    }

    pub fn set_env_var(&mut self, var: EnvVariable, value: DynValue) {
        let name = var.get_name();
        self.set_global_variable(name, value);
    }

    pub fn ancestor(&self, distance:usize) -> Option<Env> {
        if distance == 0 {
            None
        } else {
            if let Some(env) = self.enclosing.clone() {
                if distance == 1 {
                    Some(env)
                } else {
                    env.borrow_mut().ancestor(distance - 1)
                }
            } else {
                None
            }
        }
    }

    pub fn set_resolved_locals(&mut self, resolved_locals: HashMap<UUID, usize>) {
        self.resolved_locals = Some(resolved_locals);
    }

    pub fn set_global(&mut self, name: String) {
        self.globals.push(name);
    }

    pub fn set_nonlocal(&mut self, name: String) {
        self.non_locals.push(name);
    }
}

pub fn make_env() -> Env {
    Rc::new(RefCell::new(Environment::new()))
}

pub fn make_env_enclosed(enclosing: Env) -> Env {
    Rc::new(RefCell::new(Environment::new_enclosed(enclosing)))
}

pub enum EnvVariable {
    NewLines,
}

impl EnvVariable {
    fn get_name(&self) -> String {
        match self {
            EnvVariable::NewLines => String::from(".new_lines"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::myton::lexer::token::{Token, TokenKind};

    use super::*;
    use super::super::types::{DynValue, TypeKind};
    use super::super::native_functions::native_clock;
    use super::super::functions::NativeFunction;

    #[test]
    fn test_get() {
        let mut env = Environment::new();
        env.set("a".to_string().to_string(), DynValue::from(1.0));
        assert!(env.get("a".to_string()).is_some());
        assert!(env.get("a".to_string()).unwrap().is_number());
        assert_eq!(env.get("a".to_string()).unwrap().as_number(), 1.0);
        assert!(env.get("b".to_string()).is_none());
    }

    #[test]
    fn test_builtin_function() {
        let value = DynValue::new_with_name(Box::new(NativeFunction{ nb_args: 0, func: native_clock }), TypeKind::NativeFunction, "clock".to_string());
        let env = make_env();
        env.borrow_mut().set("clock".to_string(), value);

        let value = env.borrow_mut().get("clock".to_string());

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

        borrowed_local.set("a".to_string(), DynValue::from(3.0));
        borrowed_local.set("b".to_string(), DynValue::from(2.0));

        assert!(borrowed_local.get("a".to_string()).is_some());
        assert!(borrowed_local.get("a".to_string()).unwrap().is_number());
        assert_eq!(borrowed_local.get("a".to_string()).unwrap().as_number(), 3.0);

        assert!(borrowed_local.get("b".to_string()).is_some());
        assert!(borrowed_local.get("b".to_string()).unwrap().is_number());
        assert_eq!(borrowed_local.get("b".to_string()).unwrap().as_number(), 2.0);

        assert!(borrowed_local.get("c".to_string()).is_some());
        assert!(borrowed_local.get("c".to_string()).unwrap().is_number());
        assert_eq!(borrowed_local.get("c".to_string()).unwrap().as_number(), 2.0);

        assert!(borrowed_local.get("d".to_string()).is_none());

        let borrowed_global = global.borrow_mut();
        assert!(borrowed_global.get("a".to_string()).is_some());
        assert!(borrowed_global.get("a".to_string()).unwrap().is_number());
        assert_eq!(borrowed_global.get("a".to_string()).unwrap().as_number(), 1.0);
    }

    #[test]
    fn test_env_var() {
        let env = make_env();

        env.borrow_mut().set_env_var(EnvVariable::NewLines, DynValue::from(1.0));

        assert_eq!(env.borrow_mut().get_env_var(EnvVariable::NewLines).as_number(), 1.0);

        let enclosed_env = make_env_enclosed(env.clone());
        let mut borrowed_enclosed_env = enclosed_env.borrow_mut();

        assert!(borrowed_enclosed_env.get_env_var(EnvVariable::NewLines).as_number() == 1.0);
        borrowed_enclosed_env.set_env_var(EnvVariable::NewLines, DynValue::from(2.0));

        assert!(env.borrow_mut().get_env_var(EnvVariable::NewLines).as_number() == 2.0);
    }

    #[test]
    fn test_ancestor() {
        let env = make_env();
        let local = make_env_enclosed(env.clone());
        let local2 = make_env_enclosed(local.clone());

        env.borrow_mut().set("a".to_string(), DynValue::from(1.0));
        local.borrow_mut().set("b".to_string(), DynValue::from(2.0));
        local.borrow_mut().set("a".to_string(), DynValue::from(3.0));

        assert!(env.borrow().ancestor(0).is_none());
        assert!(env.borrow().ancestor(1).is_none());
        assert_eq!(env.borrow().get("a".to_string()).unwrap().as_number(), 1.0);

        assert!(local.borrow().ancestor(0).is_none());
        assert!(local.borrow().ancestor(1).is_some());
        assert!(local.borrow().ancestor(1).unwrap().borrow().get("a".to_string()).is_some());
        assert_eq!(local.borrow().ancestor(1).unwrap().borrow().get("a".to_string()).unwrap().as_number(), 1.0);
        assert!(local.borrow().ancestor(2).is_none());

        assert!(local2.borrow().ancestor(0).is_none());
        assert!(local2.borrow().ancestor(1).is_some());
        assert!(local2.borrow().ancestor(1).unwrap().borrow().get("a".to_string()).is_some());
        assert_eq!(local2.borrow().ancestor(1).unwrap().borrow().get("a".to_string()).unwrap().as_number(), 3.0);
        assert!(local2.borrow().ancestor(1).unwrap().borrow().get("b".to_string()).is_some());
        assert!(local2.borrow().ancestor(2).is_some());
        assert!(local2.borrow().ancestor(2).unwrap().borrow().get("a".to_string()).is_some());
        assert_eq!(local2.borrow().ancestor(2).unwrap().borrow().get("a".to_string()).unwrap().as_number(), 1.0);
        assert!(local2.borrow().ancestor(2).unwrap().borrow().get("b".to_string()).is_none());
    }

    #[test]
    fn test_get_from_variable() {
        let env = make_env();
        let token = Token {
            kind: TokenKind::Identifier,
            value: "a".to_string(),
            ..Default::default()
        };
        let var = Variable::new(token, 0);

        let resolved_locals = HashMap::from_iter(vec![(0, 1)].into_iter());

        env.borrow_mut().set_resolved_locals(resolved_locals);

        let local = make_env_enclosed(env.clone());

        local.borrow_mut().set("a".to_string(), DynValue::from(1.0));
        env.borrow_mut().set("a".to_string(), DynValue::from(2.0));

        assert!(local.borrow().get_from_variable(&var).is_some());
        assert_eq!(local.borrow().get_from_variable(&var).unwrap().as_number(), 2.0);
    }
}
