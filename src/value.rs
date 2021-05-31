#[derive(Copy, Clone)]
pub enum Value {
  Bool(bool),
  Number(f64),
  Nil,
}

impl PartialEq for Value {
  fn eq(&self, other: &Value) -> bool {
    match (self, other) {
      (&Value::Bool(a), &Value::Bool(b)) => a == b,
      (&Value::Nil, &Value::Nil) => true,
      (&Value::Number(a), &Value::Number(b)) => a == b,
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
      Value::Number(number) => print!("{}", number),
      Value::Nil => print!("nil"),
    }
  }
}
