use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::functions::{Callable, Function};
use super::types::DynValue;
use super::traceback::Traceback;
use super::environment::Env;

#[derive(Clone)]
pub struct Class {
    pub name: String,
    pub methods: HashMap<String, Function>,
}

pub struct Instance {
    pub class: Class,
    pub fields: Rc<RefCell<HashMap<String, DynValue>>>,
}

impl Class {
    pub fn new(name: String, methods: HashMap<String, Function>) -> Self {
        Self {
            name,
            methods,
        }
    }

    fn find_method(&self, name: &str) -> Option<&Function> {
        self.methods.get(name)
    }
}

impl Instance {
    pub fn new(class: Class) -> Self {
        Self {
            class,
            fields: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn get(&self, name: &str) -> Option<DynValue> {
        if let Some(val) = self.fields.borrow().get(name) {
            Some(val.clone())
        } else if let Some(method) = self.class.find_method(name) {
            Some(DynValue::from(method.bind(DynValue::from(self.clone()))))
        } else {
            None
        }
    }

    pub fn set(&mut self, name: String, value: DynValue) {
        self.fields.borrow_mut().insert(name, value);
    }
}

impl Clone for Instance {
    fn clone(&self) -> Self {
        Self {
            class: self.class.clone(),
            fields: self.fields.clone(),
        }
    }
}

impl Callable for Class {
    fn call(&self, env: &Env, args: Vec<DynValue>) -> Result<DynValue, Traceback> {
        let instance = Instance::new(self.clone());

        if let Some(initializer) = self.find_method("__init__"){
            initializer.bind(DynValue::from(instance.clone())).call(env, args)?;
        }


        Ok(DynValue::from(instance))
    }

    fn arity(&self) -> usize {
        if let Some(initializer) = self.find_method("__init__") {
            initializer.arity()
        } else {
            0
        }
    }
}
