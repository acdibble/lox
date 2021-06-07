use crate::chunk::Chunk;
use crate::native;
use crate::string;
use std::rc::Rc;

#[derive(Clone)]
pub struct Function {
    pub arity: usize,
    pub chunk: Rc<Chunk>,
    pub name: string::Handle,
    pub upvalue_count: usize,
}

impl Function {
    pub fn get_name(&self) -> &'static str {
        match self.name.as_str().string {
            "" => "script",
            value => value,
        }
    }

    pub fn print(&self) {
        match self.get_name() {
            "script" => print!("<script>"),
            name => print!("<fn {}>", name),
        }
    }
}

#[derive(Clone)]
pub struct Closure {
    pub function: Function,
    pub upvalues: Vec<Upvalue>,
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

#[derive(Copy, Clone)]
pub struct Upvalue {
    pub location: *const Value,
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
    Upvalue(Upvalue),
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
            Value::Upvalue(_) => print!("upvalue"),
            Value::Nil => print!("nil"),
        }
    }

    pub fn println(&self) {
        self.print();
        println!("");
    }
}
