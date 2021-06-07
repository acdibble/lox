use crate::value::*;
use std::time::{SystemTime, UNIX_EPOCH};

pub type Function = fn(args: &[Value]) -> Value;

pub fn clock(_args: &[Value]) -> Value {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs_f64();
    Value::Number(since_the_epoch)
}
