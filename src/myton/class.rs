use super::functions::Callable;
use super::types::DynValue;
use super::traceback::Traceback;
use super::environment::Env;

pub struct Class {
    pub name: String,
}

pub struct Instance {
    pub class: Class,
}

impl Class {
    pub fn new(name: String) -> Self {
        Self {
            name,
        }
    }
}

impl Clone for Class {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
        }
    }
}

impl Instance {
    pub fn new(class: Class) -> Self {
        Self {
            class,
        }
    }
}

impl Clone for Instance {
    fn clone(&self) -> Self {
        Self {
            class: self.class.clone(),
        }
    }
}

impl Callable for Class {
    fn call(&self, env: &Env, args: Vec<DynValue>) -> Result<DynValue, Traceback> {
        let instance = Instance::new(self.clone());
        Ok(DynValue::from(instance))
    }

    fn arity(&self) -> usize {
        0
    }
}
