use crate::value::Value;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct Memory {
    locals: HashMap<String, Value>,
    slots: HashMap<String, Value>,
}

impl Memory {
    pub fn set_local(&mut self, name: &str, value: Value) {
        self.locals.insert(name.to_string(), value);
    }

    pub fn get_local(&self, name: &str) -> Option<&Value> {
        self.locals.get(name)
    }

    pub fn alloc_slot(&mut self, name: &str) {
        self.slots.entry(name.to_string()).or_insert(Value::I32(0));
    }

    pub fn store_slot(&mut self, name: &str, value: Value) -> bool {
        if let Some(slot) = self.slots.get_mut(name) {
            *slot = value;
            true
        } else {
            false
        }
    }

    pub fn load_slot(&self, name: &str) -> Option<Value> {
        self.slots.get(name).cloned()
    }
}
