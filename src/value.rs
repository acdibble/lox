use crate::string;

#[derive(Clone)]
pub enum Value {
  Bool(bool),
  Number(f64),
  Nil,
  String(string::Handle),
}

impl PartialEq for Value {
  fn eq(&self, other: &Value) -> bool {
    match (self, other) {
      (Value::Bool(a), Value::Bool(b)) => a == b,
      (Value::Nil, Value::Nil) => true,
      (Value::Number(a), Value::Number(b)) => a == b,
      (Value::String(a), Value::String(b)) => a == b,
      _ => false,
    }
  }
}

impl Value {
  pub fn is_falsy(&self) -> bool {
    match self {
      Value::Nil | Value::Bool(false) => true,
      _ => false,
    }
  }

  pub fn print(&self) {
    match self {
      Value::Bool(value) => print!("{}", value),
      Value::Number(value) => print!("{}", value),
      Value::String(value) => print!("{}", value),
      Value::Nil => print!("nil"),
    }
  }
}
