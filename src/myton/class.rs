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
    pub superclass: Option<Box<Class>>,
}

#[derive(Clone)]
pub struct Instance {
    pub class: Class,
    pub fields: Rc<RefCell<HashMap<String, DynValue>>>,
}

impl Class {
    pub fn new(name: String, methods: HashMap<String, Function>, superclass: Option<Class>) -> Self {
        Self {
            name,
            methods,
            superclass: superclass.map(|c| Box::new(c)),
        }
    }

    pub fn find_method(&self, name: &str) -> Option<&Function> {
        if let Some(method) = self.methods.get(name) {
            Some(method)
        } else if let Some(superclass) = &self.superclass {
            superclass.find_method(name)
        } else {
            None
        }
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
            let refcell = Rc::new(RefCell::new(self.clone()));
            Some(DynValue::from(method.bind(refcell)))
        } else {
            None
        }
    }

    pub fn set(&mut self, name: String, value: DynValue) {
        self.fields.borrow_mut().insert(name, value);
    }
}

pub fn get_from_refcell(instance: Rc<RefCell<Instance>>, name: &str) -> Option<DynValue> {
    if let Some(val) = instance.borrow().fields.borrow().get(name) {
        Some(val.clone())
    } else if let Some(method) = instance.borrow().class.find_method(name) {
        Some(DynValue::from(method.bind(instance.clone())))
    } else {
        None
    }
}


impl Callable for Class {
    fn call(&self, env: &Env, args: Vec<DynValue>) -> Result<DynValue, Traceback> {
        let refcell = Rc::new(RefCell::new(Instance::new(self.clone())));

        if let Some(initializer) = self.find_method("__init__"){
            initializer.bind(refcell.clone()).call(env, args)?;
        }


        Ok(DynValue::from(refcell))
    }

    fn arity(&self) -> usize {
        if let Some(initializer) = self.find_method("__init__") {
            initializer.arity()
        } else {
            0
        }
    }
}
