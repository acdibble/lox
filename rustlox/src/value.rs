use crate::chunk::Chunk;
use crate::native;
use crate::string;
use std::cell::RefCell;
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Location {
    Stack(usize),
    Here,
}

impl Location {
    pub fn as_usize(&self) -> usize {
        match self {
            Location::Stack(index) => *index,
            _ => panic!(),
        }
    }
}

#[derive(Clone)]
pub struct Upvalue {
    location: Location,
    pub next: Option<Rc<RefCell<Upvalue>>>,
    pub closed: Option<Value>,
}

impl Upvalue {
    pub fn new_closed(value: Value) -> Upvalue {
        Upvalue {
            location: Location::Here,
            next: None,
            closed: Some(value),
        }
    }

    pub fn new_open(location: usize, next: Option<Rc<RefCell<Upvalue>>>) -> Upvalue {
        Upvalue {
            location: Location::Stack(location),
            next,
            closed: None,
        }
    }

    pub fn get_location(&self) -> usize {
        match self.location {
            Location::Stack(val) => val,
            _ => 0,
        }
    }

    pub fn set_location(&mut self, location: (usize, Value)) {
        if self.location == Location::Here {
            self.closed = Some(location.1.clone());
        } else {
            self.location = Location::Stack(location.0);
        }
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
