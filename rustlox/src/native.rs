use crate::value::*;
use std::time::{SystemTime, UNIX_EPOCH};

pub type Function = fn(args: &[Value]) -> Value;

pub fn clock(_args: &[Value]) -> Value {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs_f64();
    Value::Number(timestamp)
}
