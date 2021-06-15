use crate::value::*;
use std::result::Result;
use std::time::{SystemTime, SystemTimeError, UNIX_EPOCH};

pub type Function = fn(args: &[Value]) -> Value;

pub fn clock(_args: &[Value]) -> Result<f64, SystemTimeError> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs_f64())
}
