#[derive(Copy, Clone)]
pub enum Value {
  Bool(bool),
  Number(f64),
  Nil,
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
