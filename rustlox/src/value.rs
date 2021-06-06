use crate::chunk::Chunk;
use crate::string;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct Function {
    pub arity: usize,
    pub chunk: Rc<Chunk>,
    pub name: string::Handle,
}

#[derive(Clone, Debug)]
pub enum Value {
    Bool(bool),
    Number(f64),
    Nil,
    String(string::Handle),
    Function(Function),
}

impl PartialEq for Value {
    fn eq(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Nil, Value::Nil) => true,
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Function(a), Value::Function(b)) => Rc::ptr_eq(&a.chunk, &b.chunk),
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
            Value::Function(function) => match function.name.as_str().string {
                "" => print!("script"),
                name => print!("<fn {}>", name),
            },
            Value::Nil => print!("nil"),
        }
    }

    pub fn println(&self) {
        self.print();
        println!("");
    }
}
