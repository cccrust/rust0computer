#![allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Value {
    I1(bool),
    I32(i32),
    I64(i64),
    Ptr(String),
    // TODO: float, pointer, aggregates
}

impl Value {
    pub fn as_i32(&self) -> Option<i32> {
        match self {
            Value::I32(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_i1(&self) -> Option<bool> {
        match self {
            Value::I1(v) => Some(*v),
            _ => None,
        }
    }
}
