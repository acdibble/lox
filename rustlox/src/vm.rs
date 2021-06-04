use crate::chunk::*;
use crate::compiler::*;
use crate::value::*;
use std::collections::HashMap;
use std::convert::TryInto;

const CALL_FRAME_MAX: usize = 64;

#[derive(Copy, Clone)]
struct CallFrame {
    function: Function,
    ip: usize,
    starts_at: usize,
}

impl CallFrame {
    fn new(function: Function, starts_at: usize) -> CallFrame {
        CallFrame {
            function,
            starts_at,
            ip: 0,
        }
    }
}

#[derive(PartialEq)]
pub enum InterpretError {
    CompileError,
    RuntimeError,
    InternalError(&'static str),
}

pub struct VM {
    stack: Vec<Value>,
    globals: HashMap<&'static str, Value>,
    function: Option<Function>,

    frames: [Option<CallFrame>; CALL_FRAME_MAX],
    frame_count: usize,
}

type Result<T> = std::result::Result<T, InterpretError>;

impl VM {
    pub fn new() -> VM {
        VM {
            stack: Default::default(),
            globals: Default::default(),
            function: Default::default(),

            frames: [None; CALL_FRAME_MAX],
            frame_count: Default::default(),
        }
    }

    fn reset_stack(&mut self) {
        self.stack.clear()
    }

    fn current_chunk(&mut self) -> &Chunk {
        self.function.as_ref().unwrap().chunk.get_chunk()
    }

    fn runtime_error<'a>(&mut self, string: &'a str) -> Result<()> {
        eprintln!("{}", string);

        let frame = self.frames[self.frame_count - 1].as_ref().unwrap();
        let line = frame.function.chunk.get_chunk().lines[frame.ip];
        eprintln!("[line {}] in script", line);
        self.reset_stack();
        Err(InterpretError::RuntimeError)
    }

    pub fn interpret(&mut self, source: &String) -> Result<()> {
        let function = compile(source)?;
        self.function = Some(function);
        self.push(Value::Function(function));
        self.frames[self.frame_count] = Some(CallFrame::new(function, 0));
        self.frame_count += 1;
        self.run()
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value)
    }

    fn pop(&mut self) -> Result<Value> {
        match self.stack.pop() {
            Some(value) => Ok(value),
            _ => Err(InterpretError::InternalError("Can't pop on empty stack.")),
        }
    }

    fn peek(&self, index: usize) -> Result<&Value> {
        let len = self.stack.len();
        match self.stack.get(len - 1 - index) {
            Some(value) => Ok(value),
            None => Err(InterpretError::InternalError("Can't peek on empty stack.")),
        }
    }

    fn run(&mut self) -> Result<()> {
        let frame = &mut self.frames[self.frame_count - 1].unwrap();

        macro_rules! read_u8 {
            () => {{
                let ip = frame.ip;
                frame.ip += 1;
                let chunk = self.current_chunk();
                match chunk.code.get(ip) {
                    Some(byte) => Ok(*byte),
                    _ => return Err(InterpretError::InternalError("Failed to read byte.")),
                }
            }};
        }

        macro_rules! read_constant {
            () => {{
                let constant: usize = read_u8!()?.into();
                match self
                    .current_chunk()
                    .constants
                    .get(constant + frame.starts_at)
                {
                    Some(value) => Ok(value),
                    _ => return Err(InterpretError::InternalError("Failed to read constant.")),
                }
            }};
        }

        macro_rules! read_u16 {
            () => {{
                let byte1: u16 = read_u8!()?.into();
                let byte2: u16 = read_u8!()?.into();
                Ok((byte1 << 8) | byte2)
            }};
        }

        macro_rules! read_string {
            () => {
                match read_constant!()? {
                    Value::String(handle) => Ok(handle),
                    _ => return Err(InterpretError::InternalError("Value was not a string.")),
                }
            };
        }

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
                for value in self.stack.iter() {
                    print!("[ ");
                    value.print();
                    print!(" ]");
                }
                println!("");
                let ip = frame.ip;
                self.current_chunk().disassemble_instruction(ip);
            }

            let instruction = match read_u8!()?.try_into() {
                Ok(op) => op,
                Err(value) => {
                    let message = format!("Got unexpected instruction: '{}'", value);
                    return self.runtime_error(message.as_str());
                }
            };

            match instruction {
                Op::Constant => {
                    let constant = read_constant!()?;
                    {
                        #![cfg(feature = "trace-execution")]
                        constant.println();
                    }
                    let copy = *constant;
                    self.push(copy);
                }
                Op::Nil => self.push(Value::Nil),
                Op::True => self.push(Value::Bool(true)),
                Op::False => self.push(Value::Bool(false)),
                Op::Pop => {
                    self.pop()?;
                }
                Op::GetLocal => {
                    let slot: usize = read_u8!()?.into();
                    self.push(self.stack[slot]);
                }
                Op::SetLocal => {
                    let slot: usize = read_u8!()?.into();
                    self.stack[slot] = *self.peek(0)?;
                }
                Op::GetGlobal => {
                    let name = read_string!()?.as_str().string;
                    match self.globals.get(name) {
                        Some(value) => {
                            let copy = *value;
                            self.push(copy);
                        }
                        _ => {
                            let error = format!("Undefined variable '{}'.", name);
                            return self.runtime_error(error.as_str());
                        }
                    }
                }
                Op::DefineGlobal => {
                    let name = read_string!()?.as_str().string;
                    self.globals.insert(name, *self.peek(0)?);
                    self.pop()?;
                }
                Op::SetGlobal => {
                    let name = read_string!()?;
                    let string = name.as_str().string;
                    if self.globals.insert(string, *self.peek(0)?).is_none() {
                        self.globals.remove(string);
                        let error = format!("Undefined variable '{}'.", string);
                        return self.runtime_error(error.as_str());
                    }
                }
                Op::Equal => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::Bool(a == b));
                }
                Op::Greater => binary_op!(>, Bool),
                Op::Less => binary_op!(<, Bool),
                Op::Add => {
                    let value = match (self.peek(0)?, self.peek(1)?) {
                        (Value::Number(b), Value::Number(a)) => Value::Number(a + b),
                        (Value::String(b), Value::String(a)) => Value::String(a + b),
                        _ => {
                            return self.runtime_error("Operands must be numbers.");
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
                    let num = match self.peek(0)? {
                        Value::Number(num) => *num,
                        _ => {
                            return self.runtime_error("Operand must be a number.");
                        }
                    };
                    self.pop()?;
                    self.push(Value::Number(-num))
                }
                Op::Print => {
                    self.pop()?.println();
                }
                Op::Jump => {
                    let offset: usize = read_u16!()?.into();
                    frame.ip += offset;
                }
                Op::JumpIfFalse => {
                    let offset: usize = read_u16!()?.into();
                    if self.peek(0)?.is_falsy() {
                        frame.ip += offset
                    }
                }
                Op::Loop => {
                    let offset = read_u16!()?;
                    frame.ip -= offset as usize;
                }
                Op::Return => {
                    return Ok(());
                }
            }
        }
    }
}
