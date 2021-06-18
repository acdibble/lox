use crate::chunk::*;
use crate::compiler::*;
use crate::native;
use crate::string;
use crate::value::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::TryInto;
use std::rc::Rc;

fn with_vm<T, F: FnOnce(&mut VM) -> T>(f: F) -> T {
    thread_local!(static STATIC_VM: RefCell<VM> = {
        RefCell::new(VM::new())
    });
    STATIC_VM.with(|vm| f(&mut *vm.borrow_mut()))
}

#[derive(Default)]
struct CallFrame {
    closure: Option<Closure>,
    ip: usize,
    starts_at: usize,
}

impl CallFrame {
    pub fn reset(&mut self) {
        self.closure = None;
        self.starts_at = 0;
        self.ip = 0;
    }
}

#[derive(PartialEq)]
pub enum InterpretError {
    CompileError,
    RuntimeError,
    InternalError(&'static str),
}

const CALL_FRAME_MAX: usize = 64;
const CALL_FRAME_DEFAULT: CallFrame = CallFrame {
    closure: None,
    ip: 0,
    starts_at: 0,
};
const STACK_MAX: usize = 256;
const STACK_DEFAULT: Value = Value::Nil;

pub struct VM {
    globals: HashMap<&'static str, Value>,

    stack: [Value; STACK_MAX],
    stack_count: usize,

    frames: [CallFrame; CALL_FRAME_MAX],
    frame_count: usize,

    open_upvalues: Option<Rc<RefCell<Upvalue>>>,
}

type Result<T> = std::result::Result<T, InterpretError>;

pub fn interpret(source: &String) -> Result<()> {
    with_vm(|vm| {
        let closure = Closure::new(compile(source)?);
        vm.push(Value::Closure(closure.clone()));
        vm.call(closure, 0).ok();
        vm.run()
    })
}

impl VM {
    pub fn new() -> VM {
        let mut vm: VM = VM {
            globals: Default::default(),

            stack_count: Default::default(),
            stack: [STACK_DEFAULT; STACK_MAX],

            frame_count: Default::default(),
            frames: [CALL_FRAME_DEFAULT; CALL_FRAME_MAX],

            open_upvalues: Default::default(),
        };

        vm.define_native("clock", native::clock);

        vm
    }

    fn reset_stack(&mut self) {
        self.stack_count = 0;
        for frame in self.frames.iter_mut() {
            frame.reset();
        }
        self.frame_count = 0;
    }

    #[inline(always)]
    fn current_frame(&self) -> &CallFrame {
        &self.frames[self.frame_count - 1]
    }

    #[inline(always)]
    fn current_frame_mut(&mut self) -> &mut CallFrame {
        &mut self.frames[self.frame_count - 1]
    }

    #[inline(always)]
    fn current_chunk(&self) -> &Chunk {
        &self
            .current_frame()
            .closure
            .as_ref()
            .unwrap()
            .function
            .chunk
    }

    fn runtime_error<'a>(&mut self, string: &'a str) -> Result<()> {
        eprintln!("{}", string);

        for frame in self.frames[0..self.frame_count].iter().rev() {
            let function = &frame.closure.as_ref().unwrap().function;
            let line = function.chunk.lines[frame.ip - 1];

            eprint!("[line {}] in ", line);
            match function.get_name() {
                "<script>" => eprintln!("script"),
                name => eprintln!("{}()", name),
            }
        }
        self.reset_stack();
        Err(InterpretError::RuntimeError)
    }

    fn define_native(&mut self, name: &'static str, function: native::Function) {
        self.globals.insert(name, Value::Native(function));
    }

    #[inline(always)]
    fn push(&mut self, value: Value) {
        self.stack[self.stack_count] = value;
        self.stack_count += 1;
    }

    #[inline(always)]
    fn pop(&mut self) -> Result<Value> {
        if self.stack_count == 0 {
            return Err(InterpretError::InternalError("Can't pop on empty stack."));
        }

        self.stack_count -= 1;
        Ok(std::mem::take(&mut self.stack[self.stack_count]))
    }

    #[inline(always)]
    fn peek(&self, index: usize) -> Result<&Value> {
        match self.stack.get(self.stack_count - 1 - index) {
            Some(value) => Ok(value),
            None => Err(InterpretError::InternalError("Can't peek on empty stack.")),
        }
    }

    #[inline(always)]
    fn call(&mut self, closure: Closure, arg_count: usize) -> Result<()> {
        if arg_count != closure.function.arity {
            return self.runtime_error(
                format!(
                    "Expected {} arguments but got {}.",
                    closure.function.arity, arg_count
                )
                .as_str(),
            );
        }

        let starts_at = self.stack_count - arg_count - 1;
        let frame = &mut self.frames[self.frame_count];
        frame.starts_at = starts_at;
        frame.closure = Some(closure);
        frame.ip = 0;
        self.frame_count += 1;

        if self.frame_count == STACK_MAX {
            return self.runtime_error("Stack overflow.");
        }

        Ok(())
    }

    #[inline(always)]
    fn call_native(&mut self, function: native::Function, arg_count: usize) -> Result<()> {
        let arg_start = self.stack_count - arg_count - 1;
        let result = function(&self.stack[arg_start..]);
        self.stack_count -= arg_count;
        self.stack[self.stack_count - 1] = result;
        Ok(())
    }

    #[inline(always)]
    fn call_value(&mut self, callee: Value, arg_count: usize) -> Result<()> {
        match callee {
            Value::Closure(closure) => self.call(closure, arg_count),
            Value::Native(function) => self.call_native(function, arg_count),
            _ => self.runtime_error("Can only call functions and classes."),
        }
    }

    #[inline(always)]
    fn capture_upvalue(&mut self, location: *const Value) -> Rc<RefCell<Upvalue>> {
        let mut previous: Option<Rc<RefCell<Upvalue>>> = None;
        let mut current: &mut Option<Rc<RefCell<Upvalue>>> = &mut self.open_upvalues;
        let mut _temp: Option<Rc<RefCell<Upvalue>>> = None;
        while current.is_some()
            && current.as_ref().unwrap().borrow().location as usize > location as usize
        {
            previous = Some(Rc::clone(&current.as_ref().unwrap()));
            _temp = if let Some(value) = &previous.as_ref().unwrap().borrow().next {
                Some(Rc::clone(value))
            } else {
                None
            };
            current = &mut _temp;
        }

        if let Some(value) = current {
            let upvalue = value.borrow();
            if upvalue.location as usize == location as usize {
                return Rc::clone(&value);
            }
        }

        let created_upvalue = Rc::new(RefCell::new(Upvalue::new(
            location,
            match current {
                Some(value) => Some(Rc::clone(value)),
                None => None,
            },
        )));

        if previous.is_none() {
            self.open_upvalues = Some(Rc::clone(&created_upvalue));
        } else {
            previous.as_mut().unwrap().borrow_mut().next = Some(Rc::clone(&created_upvalue));
        }

        created_upvalue
    }

    #[inline(always)]
    fn close_upvalues(&mut self, last: *const Value) {
        while self.open_upvalues.is_some()
            && self.open_upvalues.as_ref().unwrap().borrow().location as usize >= last as usize
        {
            let rc = self.open_upvalues.as_ref().unwrap();
            let mut upvalue = rc.borrow_mut();
            let next = match &upvalue.next {
                Some(value) => Some(Rc::clone(value)),
                None => None,
            };
            upvalue.close();
            drop(upvalue);
            self.open_upvalues = next;
        }
    }

    #[inline(always)]
    fn read_u8(&mut self) -> Result<u8> {
        let frame = self.current_frame_mut();
        let ip = frame.ip;
        frame.ip += 1;
        let chunk = self.current_chunk();
        match chunk.code.get(ip) {
            Some(byte) => Ok(*byte),
            _ => return Err(InterpretError::InternalError("Failed to read byte.")),
        }
    }

    #[inline(always)]
    fn read_constant(&mut self) -> Result<&Value> {
        let constant: usize = self.read_u8()?.into();
        match self.current_chunk().constants.get(constant) {
            Some(value) => Ok(value),
            _ => return Err(InterpretError::InternalError("Failed to read constant.")),
        }
    }

    #[inline(always)]
    fn read_u16(&mut self) -> Result<u16> {
        let byte1: u16 = self.read_u8()?.into();
        let byte2: u16 = self.read_u8()?.into();
        Ok((byte1 << 8) | byte2)
    }

    #[inline(always)]
    fn read_string(&mut self) -> Result<&string::Handle> {
        match self.read_constant()? {
            Value::String(handle) => Ok(handle),
            _ => return Err(InterpretError::InternalError("Value was not a string.")),
        }
    }

    fn run(&mut self) -> Result<()> {
        macro_rules! binary_op {
            ($op: tt, $variant: ident) => {{
                let value = match (self.peek(0)?, self.peek(1)?) {
                (Value::Number(b), Value::Number(a)) => (a $op b),
                _ => {
                    return self.runtime_error("Operands must be numbers.");
                }
                };

                self.pop()?;
                self.pop()?;
                self.push(Value::$variant(value))
            }};
        }

        loop {
            {
                #![cfg(feature = "trace-execution")]
                print!("          ");
                for i in 0..self.stack_count {
                    print!("[ ");
                    self.stack[i].print();
                    print!(" ]");
                }
                println!("");
                let ip = self.current_frame().ip;
                self.current_chunk().disassemble_instruction(ip);
            }

            let instruction = match self.read_u8()?.try_into() {
                Ok(op) => op,
                Err(value) => {
                    let message = format!("Got unexpected instruction: '{}'", value);
                    return self.runtime_error(message.as_str());
                }
            };

            match instruction {
                Op::Constant => {
                    let constant = self.read_constant()?.clone();
                    self.push(constant);
                }
                Op::Nil => self.push(Value::Nil),
                Op::True => self.push(Value::Bool(true)),
                Op::False => self.push(Value::Bool(false)),
                Op::Pop => {
                    self.pop()?;
                }
                Op::GetLocal => {
                    let slot: usize = self.read_u8()?.into();
                    let offset = self.current_frame().starts_at;
                    self.push(self.stack[slot + offset].clone());
                }
                Op::SetLocal => {
                    let slot: usize = self.read_u8()?.into();
                    let offset = self.current_frame().starts_at;
                    self.stack[slot + offset] = self.peek(0)?.clone();
                }
                Op::GetGlobal => {
                    let name = self.read_string()?.as_str().string;
                    match self.globals.get(name) {
                        Some(value) => {
                            let clone = value.clone();
                            self.push(clone);
                        }
                        _ => {
                            let error = format!("Undefined variable '{}'.", name);
                            return self.runtime_error(error.as_str());
                        }
                    }
                }
                Op::DefineGlobal => {
                    let name = self.read_string()?.as_str().string;
                    let value = self.pop()?;
                    self.globals.insert(name, value);
                }
                Op::SetGlobal => {
                    let name = self.read_string()?;
                    let string = name.as_str().string;
                    if self.globals.insert(string, self.peek(0)?.clone()).is_none() {
                        self.globals.remove(string);
                        let error = format!("Undefined variable '{}'.", string);
                        return self.runtime_error(error.as_str());
                    }
                }
                Op::GetUpvalue => {
                    let slot = self.read_u8()? as usize;
                    let value = self.current_frame().closure.as_ref().unwrap().upvalues[slot]
                        .borrow()
                        .as_value();
                    self.push(value)
                }
                Op::SetUpvalue => {
                    let slot = self.read_u8()? as usize;
                    let value = self.peek(0)?.clone();
                    let mut upvalue = self.current_frame_mut().closure.as_mut().unwrap().upvalues
                        [slot]
                        .borrow_mut();

                    upvalue.closed = value;
                }
                Op::Equal => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(Value::Bool(a == b));
                }
                Op::Greater => binary_op!(>, Bool),
                Op::Less => binary_op!(<, Bool),
                Op::Add => {
                    let value = match (self.peek(0)?, self.peek(1)?) {
                        (Value::Number(b), Value::Number(a)) => Value::Number(a + b),
                        (Value::String(b), Value::String(a)) => Value::String(a + b),
                        _ => {
                            return self
                                .runtime_error("Operands must be two numbers or two strings.");
                        }
                    };

                    self.pop()?;
                    self.pop()?;
                    self.push(value);
                }
                Op::Subtract => binary_op!(-, Number),
                Op::Multiply => binary_op!(*, Number),
                Op::Divide => binary_op!(/, Number),
                Op::Not => {
                    let value = self.pop()?.is_falsy();
                    self.push(Value::Bool(value))
                }
                Op::Negate => {
                    let num = match self.pop()? {
                        Value::Number(num) => num,
                        _ => {
                            return self.runtime_error("Operand must be a number.");
                        }
                    };
                    self.push(Value::Number(-num))
                }
                Op::Print => {
                    self.pop()?.println();
                }
                Op::Jump => {
                    let offset: usize = self.read_u16()?.into();
                    let mut frame = self.current_frame_mut();
                    frame.ip += offset;
                }
                Op::JumpIfFalse => {
                    let offset: usize = self.read_u16()?.into();
                    if self.peek(0)?.is_falsy() {
                        let frame = self.current_frame_mut();
                        frame.ip += offset
                    }
                }
                Op::Loop => {
                    let offset = self.read_u16()?;
                    let frame = self.current_frame_mut();
                    frame.ip -= offset as usize;
                }
                Op::Call => {
                    let arg_count = self.read_u8()? as usize;
                    let callee = self.peek(arg_count)?.clone();
                    self.call_value(callee, arg_count)?;
                }
                Op::Closure => {
                    let fun = match self.read_constant()? {
                        Value::Function(fun) => Ok(fun.clone()),
                        _ => Err(InterpretError::InternalError(
                            "Expected function for closure",
                        )),
                    }?;
                    let upvalue_count = fun.upvalue_count;
                    let mut closure = Closure::new(fun);
                    let offset = self.current_frame().starts_at;
                    for i in 0..upvalue_count {
                        let is_local = self.read_u8()?;
                        let index = self.read_u8()? as usize;
                        let upvalue = if is_local == 1 {
                            self.capture_upvalue(&self.stack[offset + index])
                        } else {
                            self.current_frame().closure.as_ref().unwrap().upvalues[i].clone()
                        };
                        closure.upvalues.push(upvalue)
                    }
                    self.push(Value::Closure(closure));
                }
                Op::CloseUpvalue => {
                    self.close_upvalues(&self.stack[self.stack_count - 1]);
                    self.pop()?;
                }
                Op::Return => {
                    let result = self.pop()?;
                    let starts_at = self.current_frame().starts_at;
                    self.close_upvalues(&self.stack[starts_at]);
                    self.frame_count -= 1;
                    if self.frame_count == 0 {
                        self.pop()?;
                        return Ok(());
                    }

                    self.stack_count = starts_at;
                    self.push(result)
                }
            }
        }
    }
}
