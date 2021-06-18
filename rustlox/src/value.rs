use crate::chunk::Chunk;
use crate::native;
use crate::string;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct Function {
    pub arity: usize,
    pub chunk: Rc<Chunk>,
    pub name: string::Handle,
    pub upvalue_count: usize,
}

impl Function {
    pub fn get_name(&self) -> &'static str {
        match self.name.as_str().string {
            "" => "<script>",
            value => value,
        }
    }

    pub fn print(&self) {
        match self.get_name() {
            "<script>" => print!("<script>"),
            name => print!("<fn {}>", name),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Closure {
    pub function: Function,
    pub upvalues: Vec<Rc<RefCell<Upvalue>>>,
    pub upvalue_count: usize,
}

impl Closure {
    pub fn new(function: Function) -> Closure {
        Closure {
            upvalue_count: function.upvalue_count,
            upvalues: Vec::with_capacity(function.upvalue_count),
            function,
        }
    }
}

impl Drop for Closure {
    #![cfg(feature = "debug-drop")]
    fn drop(&mut self) {
        println!("cya!: {:?}", self.function.get_name());
    }
}

#[derive(Clone, Debug)]
pub struct Upvalue {
    pub location: *mut Value,
    pub next: Option<Rc<RefCell<Upvalue>>>,
    pub closed: Value,
}

impl Drop for Upvalue {
    #![cfg(feature = "debug-drop")]
    fn drop(&mut self) {
        println!("cya!: {:?}", self.closed);
    }
}

impl Upvalue {
    pub fn new(location: *mut Value, next: Option<Rc<RefCell<Upvalue>>>) -> Upvalue {
        Upvalue {
            location,
            next,
            closed: Value::Nil,
        }
    }

    pub fn close(&mut self) {
        unsafe { self.closed = (*self.location).clone() };
        self.location = &mut self.closed;
        self.next = None;
    }

    pub fn as_value(&self) -> Value {
        unsafe { (*self.location).clone() }
    }

    pub fn set_value(&mut self, value: Value) {
        unsafe { *self.location = value }
    }
}

#[derive(Clone)]
pub enum Value {
    Bool(bool),
    Number(f64),
    Nil,
    String(string::Handle),
    Function(Function),
    Native(native::Function),
    Closure(Closure),
}

impl Default for Value {
    fn default() -> Self {
        Value::Nil
    }
}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Value::Bool(value) => write!(f, "Value::Bool({})", value),
            Value::Number(value) => write!(f, "Value::Number({})", value),
            Value::Nil => write!(f, "Value::Nil"),
            Value::String(value) => write!(f, "Value::String({})", value),
            Value::Function(value) => write!(f, "Value::Function({:?})", value),
            Value::Native(_) => write!(f, "Value::Native(<native fn>)"),
            Value::Closure(value) => write!(f, "Value::Closure({:?})", value),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Nil, Value::Nil) => true,
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Function(a), Value::Function(b)) => Rc::ptr_eq(&a.chunk, &b.chunk),
            (Value::Native(a), Value::Native(b)) => *a as usize == *b as usize,
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
            Value::Function(function) => function.print(),
            Value::Native(_) => print!("<native fn>"),
            Value::Closure(closure) => closure.function.print(),
            Value::Nil => print!("nil"),
        }
    }

    pub fn println(&self) {
        self.print();
        println!("");
    }
}
