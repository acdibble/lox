use crate::chunk::*;
use crate::expr::{self, Expr};
use crate::parser;
use crate::scanner::{self, Token, TokenKind};
use crate::stmt::{self, Stmt};
use crate::string;
use crate::value::*;
use crate::vm::InterpretError;
use std::cell::RefCell;
use std::convert::TryInto;
use std::rc::Rc;

#[derive(Copy, Clone)]
struct Local<'a> {
    name: &'a str,
    depth: Option<usize>,
    is_captured: bool,
}

#[derive(Copy, Clone)]
struct Upvalue {
    index: u8,
    is_local: bool,
}

struct Compiler<'a> {
    enclosing: Option<Rc<RefCell<Compiler<'a>>>>,
    function: Function,

    locals: Vec<Local<'a>>,
    scope_depth: usize,
    upvalues: Vec<Upvalue>,
}

impl<'a> Compiler<'a> {
    fn new(enclosing: Option<Rc<RefCell<Compiler<'a>>>>, name: &str) -> Compiler<'a> {
        Compiler {
            enclosing,
            function: Function {
                arity: 0,
                chunk: Rc::new(Chunk::new()),
                name: string::Handle::from_str(name),
                upvalue_count: 0,
            },
            scope_depth: 0,
            locals: vec![Local {
                depth: Some(0),
                name: "",
                is_captured: false,
            }],
            upvalues: Vec::new(),
        }
    }
}

impl<'a> Compiler<'a> {
    fn with_enclosing<T, F: FnOnce(&Compiler) -> T>(&self, f: F) -> T {
        let enclosing = self.enclosing.as_ref().unwrap().borrow();
        f(&enclosing)
    }

    fn with_enclosing_mut<T, F: FnOnce(&mut Compiler) -> T>(&self, f: F) -> T {
        let mut enclosing = self.enclosing.as_ref().unwrap().borrow_mut();
        f(&mut enclosing)
    }

    fn resolve_local(&self, name: &str) -> (Option<u8>, Option<&'static str>) {
        for (i, local) in self.locals.iter().enumerate().rev() {
            if local.name == name {
                if local.depth.is_none() {
                    return (
                        Some(i as u8),
                        Some("Can't read local variable in its own initializer."),
                    );
                }
                return (Some(i as u8), None);
            }
        }

        (None, None)
    }

    fn add_upvalue(&mut self, index: u8, is_local: bool) -> Result<u8, &'static str> {
        for (index, upvalue) in self.upvalues.iter().enumerate() {
            if upvalue.index as usize == index && upvalue.is_local == is_local {
                return Ok(upvalue.index);
            }
        }

        self.upvalues.push(Upvalue { is_local, index });
        self.function.upvalue_count += 1;
        match (self.upvalues.len() - 1).try_into() {
            Ok(value) => Ok(value),
            _ => Err("Too many closure variables in function."),
        }
    }

    fn resolve_upvalue(&mut self, name: &str) -> (Option<u8>, Option<&'static str>) {
        if self.enclosing.is_none() {
            return (None, None);
        }

        if let (Some(local), err) = self.with_enclosing(|c| c.resolve_local(name)) {
            self.with_enclosing_mut(|c| c.locals[local as usize].is_captured = true);
            return match self.add_upvalue(local, true) {
                Ok(value) => (Some(value), err),
                Err(message) => (Some(0), Some(message)),
            };
        }

        if let (Some(value), err) = self.with_enclosing_mut(|c| c.resolve_upvalue(name)) {
            return match self.add_upvalue(value, false) {
                Ok(value) => (Some(value), err),
                Err(message) => (Some(0), Some(message)),
            };
        }

        (None, None)
    }
}

struct CompilerWrapper<'a> {
    current: Option<Rc<RefCell<Compiler<'a>>>>,
    current_line: i32,

    continue_point: usize,
    breaks: Vec<(usize, usize)>,
    loop_depth: usize,
}

impl<'a> CompilerWrapper<'a> {
    pub fn new() -> CompilerWrapper<'a> {
        CompilerWrapper {
            current: Some(Rc::new(RefCell::new(Compiler::new(None, "")))),
            current_line: 0,
            continue_point: 0,
            breaks: Vec::new(),
            loop_depth: 0,
        }
    }

    fn with_current_chunk<T, F: FnOnce(&Chunk) -> T>(&self, f: F) -> T {
        let current = self.current.as_ref().unwrap().borrow();
        f(&current.function.chunk)
    }

    fn with_current_chunk_mut<T, F: FnOnce(&mut Chunk) -> T>(&mut self, f: F) -> T {
        let mut current = self.current.as_ref().unwrap().borrow_mut();
        let chunk = Rc::get_mut(&mut current.function.chunk).unwrap();
        f(chunk)
    }

    fn with_current_function_mut<T, F: FnOnce(&mut Function) -> T>(&mut self, f: F) -> T {
        let mut current = self.current.as_ref().unwrap().borrow_mut();
        f(&mut current.function)
    }

    fn with_current<T, F: FnOnce(&Compiler) -> T>(&self, f: F) -> T {
        let current = self.current.as_ref().unwrap().borrow();
        f(&current)
    }

    fn with_current_mut<T, F: FnOnce(&mut Compiler) -> T>(&mut self, f: F) -> T {
        let mut current = self.current.as_ref().unwrap().borrow_mut();
        f(&mut current)
    }

    fn emit_byte(&mut self, byte: u8) {
        let line = self.current_line;
        self.with_current_chunk_mut(|chunk| chunk.write(byte, line))
    }

    fn emit_op(&mut self, op: Op) {
        self.emit_byte(op as u8)
    }

    fn emit_ops(&mut self, op1: Op, op2: Op) {
        self.emit_op(op1);
        self.emit_op(op2);
    }

    fn emit_bytes(&mut self, byte1: u8, byte2: u8) {
        self.emit_byte(byte1);
        self.emit_byte(byte2);
    }

    fn emit_loop(&mut self, loop_start: usize) {
        self.emit_op(Op::Loop);

        let offset: u16 = match self
            .with_current_chunk(|chunk| chunk.code.len() - loop_start + 2)
            .try_into()
        {
            Ok(val) => val,
            Err(_) => {
                self.error("Loop body too large.");
                0
            }
        };

        self.emit_byte((offset >> 8) as u8 & 0xff);
        self.emit_byte((offset & 0xff) as u8);
    }

    fn emit_jump(&mut self, instruction: Op) -> usize {
        self.emit_op(instruction);
        self.emit_byte(0xff);
        self.emit_byte(0xff);
        return self.with_current_chunk(|chunk| chunk.code.len() - 2);
    }

    fn emit_return(&mut self) {
        self.emit_op(Op::Nil);
        self.emit_op(Op::Return);
    }

    fn make_constant(&mut self, value: Value) -> u8 {
        self.with_current_chunk_mut(|chunk| chunk.add_constant(value))
    }

    fn emit_constant(&mut self, value: Value) {
        let constant = self.make_constant(value);
        self.emit_bytes(Op::Constant as u8, constant);
    }

    fn patch_jump(&mut self, offset: usize) {
        let jump: u16 = match self
            .with_current_chunk(|chunk| chunk.code.len() - offset - 2)
            .try_into()
        {
            Ok(value) => value,
            Err(_) => {
                self.error("Too much code to jump over.");
                0
            }
        };

        self.with_current_chunk_mut(|chunk| chunk.code[offset] = ((jump >> 8) & 0xff) as u8);
        self.with_current_chunk_mut(|chunk| chunk.code[offset + 1] = (jump & 0xff) as u8);
    }

    #[inline(always)]
    fn get_current_len(&self) -> usize {
        self.with_current_chunk(|chunk| chunk.code.len())
    }

    fn identifier_constant(&mut self, name: &str) -> u8 {
        self.make_constant(Value::String(string::Handle::from_str(name)))
    }

    fn add_local(&mut self, name: Token<'a>) {
        if self.current.as_ref().unwrap().borrow().locals.len() >= u8::MAX as usize {
            self.error("Too many local variables in function.");
            return;
        }

        self.current
            .as_ref()
            .unwrap()
            .borrow_mut()
            .locals
            .push(Local {
                name: name.lexeme,
                depth: None,
                is_captured: false,
            })
    }

    fn declare_variable(&mut self, name: &'a Token<'a>) {
        if self.current.as_ref().unwrap().borrow().scope_depth == 0 {
            return;
        }

        let mut unique = true;
        for local in self.current.as_ref().unwrap().borrow().locals.iter().rev() {
            if local.depth.is_some()
                && local.depth.unwrap() < self.current.as_ref().unwrap().borrow().scope_depth
            {
                break;
            }

            if name.lexeme == local.name {
                unique = false;
                break;
            }
        }

        if !unique {
            self.error("Already a variable with this name in this scope.");
        }

        self.add_local(*name);
    }

    fn parse_variable(&mut self, token: &'a Token<'a>) -> u8 {
        self.declare_variable(token);
        if self.current.as_ref().unwrap().borrow().scope_depth > 0 {
            return 0;
        }

        return self.identifier_constant(token.lexeme);
    }

    fn mark_initialized(&mut self) {
        self.with_current_mut(|current| {
            if current.scope_depth == 0 {
                return;
            }
            let depth = current.scope_depth;
            current.locals.last_mut().unwrap().depth = Some(depth);
        })
    }

    fn define_variable(&mut self, global: u8) {
        if self.current.as_ref().unwrap().borrow().scope_depth > 0 {
            self.mark_initialized();
            return;
        }

        self.emit_bytes(Op::DefineGlobal as u8, global)
    }

    fn patch_breaks(&mut self) {
        while self
            .breaks
            .last()
            .map_or(false, |(_, depth)| *depth == self.loop_depth)
        {
            let (jump, _) = self.breaks.pop().unwrap();
            self.patch_jump(jump);
        }
    }

    fn end_compiler(&mut self) -> Compiler<'a> {
        self.emit_return();
        let mut compiler = Rc::try_unwrap(std::mem::take(&mut self.current).unwrap())
            .ok()
            .unwrap()
            .into_inner();
        {
            #![cfg(feature = "trace-execution")]
            let function = &compiler.function;
            function.chunk.disassemble(function.get_name());
        }
        self.current = std::mem::take(&mut compiler.enclosing);
        compiler
    }

    fn begin_scope(&mut self) {
        self.with_current_mut(|current| current.scope_depth += 1)
    }

    fn end_scope(&mut self) {
        let ops = self.with_current_mut(|current| {
            let mut ops: Vec<Op> = Vec::new();
            current.scope_depth -= 1;

            while let Some(local) = current.locals.last() {
                if local.depth.unwrap() > current.scope_depth {
                    ops.push(if local.is_captured {
                        Op::CloseUpvalue
                    } else {
                        Op::Pop
                    });
                    current.locals.pop();
                } else {
                    break;
                }
            }

            ops
        });

        for op in ops {
            self.emit_op(op);
        }
    }

    fn compile(
        &mut self,
        statements: std::vec::IntoIter<Stmt<'a>>,
    ) -> Result<Function, InterpretError> {
        for statement in statements {
            self.statement(&statement)
        }

        let compiler = self.end_compiler();
        Ok(compiler.function)
    }

    fn error(&mut self, _message: &'static str) {
        todo!()
    }

    fn statement(&mut self, statement: &Stmt<'a>) {
        match statement {
            Stmt::Block(statement) => {
                for stmt in &statement.statements {
                    self.statement(stmt);
                }
            }
            Stmt::Break(statement) => self.break_statement(statement),
            Stmt::Continue(statement) => self.continue_statement(statement),
            Stmt::Expression(statement) => self.expression(&statement.expression),
            Stmt::For(statement) => self.for_statement(statement),
            Stmt::Function(statement) => self.fun_declaration(statement),
            Stmt::If(statement) => self.if_statement(statement),
            Stmt::Print(statement) => self.print_statement(statement),
            Stmt::Return(statement) => self.return_statement(statement),
            Stmt::While(statement) => self.while_statement(statement),
            Stmt::Var(statement) => self.var_declaration(statement),
        }
    }

    fn break_statement(&mut self, _statement: &stmt::Break) {
        let jump = self.emit_jump(Op::Jump);
        let depth = self.loop_depth;
        self.breaks.push((jump, depth))
    }

    fn continue_statement(&mut self, _statement: &stmt::Continue) {
        self.emit_loop(self.continue_point);
    }

    fn function(&mut self, function: &stmt::Function<'a>) {
        self.current = Some(Rc::new(RefCell::new(Compiler::new(
            Some(self.current.as_ref().unwrap().clone()),
            function.name.lexeme,
        ))));
        self.with_current_function_mut(|fun| fun.arity = function.params.len());
        self.begin_scope();

        for token in &function.params {
            let constant = self.parse_variable(token);
            self.define_variable(constant);
        }

        for stmt in &function.body {
            self.statement(stmt)
        }

        let compiler = self.end_compiler();
        let constant = self.make_constant(Value::Function(compiler.function));
        self.emit_bytes(Op::Closure as u8, constant);

        for Upvalue { index, is_local } in compiler.upvalues {
            self.emit_byte(is_local.into());
            self.emit_byte(index);
        }
    }

    fn for_statement(&mut self, statement: &stmt::For<'a>) {
        self.begin_scope();

        if let Some(stmt) = &statement.initializer {
            self.statement(stmt);
        }

        let mut before_condition: Option<usize> = None;
        let mut jump_after_cond: Option<usize> = None;
        let mut jump_to_body: Option<usize> = None;

        if let Some(cond) = &statement.condition {
            before_condition = Some(self.get_current_len());
            self.expression(cond);
            jump_after_cond = Some(self.emit_jump(Op::JumpIfFalse));
            self.emit_op(Op::Pop);
            jump_to_body = Some(self.emit_jump(Op::Jump));
        }

        let mut before_increment: Option<usize> = None;

        if let Some(incr) = &statement.increment {
            before_increment = Some(self.get_current_len());
            self.expression(incr);
            if let Some(loop_point) = before_condition {
                self.emit_loop(loop_point)
            }
        }

        let before_body = self.get_current_len();

        if let Some(jump) = jump_to_body {
            self.patch_jump(jump);
        }

        self.loop_depth += 1;

        let enclosing_continue_point = self.continue_point;
        self.continue_point = if let Some(incr) = before_increment {
            incr
        } else if let Some(cond) = before_condition {
            cond
        } else {
            before_body
        };

        self.statement(&statement.body);

        self.emit_loop(self.continue_point);

        if let Some(jump) = jump_after_cond {
            self.patch_jump(jump);
        }

        self.patch_breaks();
        self.continue_point = enclosing_continue_point;
        self.loop_depth -= 1;

        self.end_scope();
    }

    fn fun_declaration(&mut self, function: &stmt::Function<'a>) {
        let global = self.parse_variable(function.name);
        self.mark_initialized();
        self.function(function);
        self.define_variable(global);
    }

    fn if_statement(&mut self, statement: &stmt::If<'a>) {
        self.expression(&statement.condition);

        let jump_to_else = self.emit_jump(Op::JumpIfFalse);
        self.emit_op(Op::Pop);
        self.statement(statement.then_branch.as_ref());

        let jump_from_then = self.emit_jump(Op::Jump);
        self.patch_jump(jump_to_else);
        self.emit_op(Op::Pop);

        if let Some(stmt) = &statement.else_branch {
            self.statement(stmt.as_ref());
        }
        self.patch_jump(jump_from_then);
    }

    fn print_statement(&mut self, statement: &stmt::Print) {
        self.expression(&statement.expression);
        self.emit_op(Op::Print)
    }

    fn return_statement(&mut self, statement: &stmt::Return) {
        if let Some(value) = &statement.value {
            self.expression(value)
        } else {
            self.emit_op(Op::Nil)
        }

        self.emit_op(Op::Return)
    }

    fn while_statement(&mut self, statement: &stmt::While<'a>) {
        let enclosing_continue_point = self.continue_point;
        self.continue_point = self.get_current_len();
        self.loop_depth += 1;

        self.expression(&statement.condition);
        let end_jump = self.emit_jump(Op::JumpIfFalse);
        self.emit_op(Op::Pop);

        self.statement(statement.body.as_ref());
        self.emit_loop(self.continue_point);
        self.patch_jump(end_jump);
        self.emit_op(Op::Pop);

        self.patch_breaks();

        self.loop_depth -= 1;
        self.continue_point = enclosing_continue_point;
    }

    fn var_declaration(&mut self, statement: &stmt::Var<'a>) {
        let global = self.parse_variable(statement.name);

        if let Some(expr) = &statement.initializer {
            self.expression(expr);
        } else {
            self.emit_op(Op::Nil);
        }

        self.define_variable(global);
    }

    fn expression(&mut self, expression: &Expr) {
        match expression {
            Expr::Assign(expr) => self.assignment(expr),
            Expr::Binary(expr) => self.binary(expr),
            Expr::Call(expr) => self.call(expr),
            Expr::Literal(expr) => self.literal(expr),
            Expr::Logical(expr) => self.logical(expr),
            Expr::Unary(expr) => self.unary(expr),
            Expr::Variable(expr) => self.variable(expr),
        }
    }

    fn assignment(&mut self, assignment: &expr::Assign) {
        self.expression(assignment.value.as_ref());

        let name = assignment.name.lexeme;
        let (set_op, arg) = if let (Some(arg), err) = self.with_current(|c| c.resolve_local(name)) {
            if let Some(message) = err {
                self.error(message)
            }
            (Op::SetLocal, arg)
        } else if let (Some(arg), err) = self.with_current_mut(|c| c.resolve_upvalue(name)) {
            if let Some(message) = err {
                self.error(message)
            }
            (Op::SetUpvalue, arg)
        } else {
            (Op::SetGlobal, self.identifier_constant(name))
        };

        self.emit_bytes(set_op as u8, arg);
        self.emit_op(Op::Pop);
    }

    fn binary(&mut self, binary: &expr::Binary) {
        self.expression(binary.left.as_ref());
        self.expression(binary.right.as_ref());

        self.current_line = binary.operator.line;
        match binary.operator.kind {
            TokenKind::BangEqual => self.emit_ops(Op::Equal, Op::Not),
            TokenKind::EqualEqual => self.emit_op(Op::Equal),
            TokenKind::Greater => self.emit_op(Op::Greater),
            TokenKind::GreaterEqual => self.emit_ops(Op::Less, Op::Not),
            TokenKind::Less => self.emit_op(Op::Less),
            TokenKind::LessEqual => self.emit_ops(Op::Greater, Op::Not),
            TokenKind::Plus => self.emit_op(Op::Add),
            TokenKind::Minus => self.emit_op(Op::Subtract),
            TokenKind::Slash => self.emit_op(Op::Divide),
            TokenKind::Star => self.emit_op(Op::Multiply),
            _ => unreachable!(),
        }
    }

    fn call(&mut self, call: &expr::Call) {
        self.expression(&call.callee);
        for arg in &call.args {
            self.expression(arg);
        }
        self.emit_bytes(Op::Call as u8, call.args.len() as u8);
    }

    fn literal(&mut self, literal: &expr::Literal) {
        self.current_line = literal.value.line;
        match literal.value.kind {
            TokenKind::Nil => self.emit_op(Op::Nil),
            TokenKind::False => self.emit_op(Op::False),
            TokenKind::True => self.emit_op(Op::True),
            TokenKind::Number => self.number(literal.value.lexeme),
            TokenKind::String => self.string(literal.value.lexeme),
            _ => unreachable!(),
        }
    }

    fn logical(&mut self, logical: &expr::Logical) {
        match logical.operator.kind {
            TokenKind::And => self.and(logical),
            TokenKind::Or => self.or(logical),
            _ => unreachable!(),
        }
    }

    fn unary(&mut self, unary: &expr::Unary) {
        self.current_line = unary.operator.line;
        self.expression(unary.right.as_ref());
        match unary.operator.kind {
            TokenKind::Bang => self.emit_op(Op::Not),
            TokenKind::Minus => self.emit_op(Op::Negate),
            _ => unreachable!(),
        }
    }

    fn variable(&mut self, variable: &expr::Variable) {
        let name = variable.name.lexeme;
        let (get_op, arg) = if let (Some(arg), err) = self.with_current(|c| c.resolve_local(name)) {
            if let Some(message) = err {
                self.error(message)
            }
            (Op::GetLocal, arg)
        } else if let (Some(arg), err) = self.with_current_mut(|c| c.resolve_upvalue(name)) {
            if let Some(message) = err {
                self.error(message)
            }
            (Op::GetUpvalue, arg)
        } else {
            (Op::GetGlobal, self.identifier_constant(name))
        };

        self.emit_bytes(get_op as u8, arg);
    }

    fn and(&mut self, logical: &expr::Logical) {
        self.expression(logical.left.as_ref());
        let jump = self.emit_jump(Op::JumpIfFalse);
        self.emit_op(Op::Pop);

        self.expression(logical.right.as_ref());
        self.patch_jump(jump);
    }

    fn or(&mut self, logical: &expr::Logical) {
        self.expression(logical.left.as_ref());
        let else_jump = self.emit_jump(Op::JumpIfFalse);
        let end_jump = self.emit_jump(Op::Jump);

        self.patch_jump(else_jump);
        self.emit_op(Op::Pop);
        self.expression(logical.right.as_ref());

        self.patch_jump(end_jump);
    }

    fn number(&mut self, lexeme: &str) {
        let value: f64 = lexeme.parse().expect("Failed to parse string into float");
        self.emit_constant(Value::Number(value))
    }

    fn string(&mut self, lexeme: &str) {
        let handle = string::Handle::from_str(&lexeme[1..lexeme.len() - 1]);
        self.emit_constant(Value::String(handle))
    }
}

pub fn compile(source: &String) -> Result<Function, InterpretError> {
    let tokens = scanner::scan_tokens(source);
    let statements = parser::parse_tokens(&tokens);
    if statements.is_none() {
        return Err(InterpretError::CompileError);
    }
    let statements = statements.unwrap().into_iter();
    let mut compiler = CompilerWrapper::new();
    compiler.compile(statements)
}
