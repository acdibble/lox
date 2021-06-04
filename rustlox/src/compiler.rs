use crate::chunk::{self, Chunk, Op};
use crate::scanner::*;
use crate::string;
use crate::value::*;
use crate::vm::InterpretError;
use std::convert::TryInto;

#[derive(Copy, Clone, PartialOrd, PartialEq)]
#[repr(u8)]
enum Precedence {
    None,
    Assignment, // =
    Or,         // or
    And,        // and
    Equality,   // == !=
    Comparison, // < > <= >=
    Term,       // + -
    Factor,     // * /
    Unary,      // ! -
    Call,       // . ()
    Primary,
}

impl Precedence {
    fn higher(&self) -> Precedence {
        match self {
            Precedence::None => Precedence::Assignment,
            Precedence::Assignment => Precedence::Or,
            Precedence::Or => Precedence::And,
            Precedence::And => Precedence::Equality,
            Precedence::Equality => Precedence::Comparison,
            Precedence::Comparison => Precedence::Term,
            Precedence::Term => Precedence::Factor,
            Precedence::Factor => Precedence::Unary,
            Precedence::Unary => Precedence::Call,
            _ => Precedence::Primary,
        }
    }
}

enum ErrorLocation {
    Current,
    Previous,
    End,
}

type ParseFn<'a> = fn(&mut Compiler<'a>, bool);
type ParseRule<'a> = (Option<ParseFn<'a>>, Option<ParseFn<'a>>, Precedence);

struct Parser<'a> {
    previous: Option<Token<'a>>,
    current: Option<Token<'a>>,
    had_error: bool,
    panic_mode: bool,
}

#[derive(Copy, Clone)]
struct Local<'a> {
    name: &'a str,
    depth: Option<usize>,
}

enum FunctionType {
    Function,
    Script,
}

pub struct Compiler<'a> {
    current_function: Function,
    function_type: FunctionType,

    parser: Parser<'a>,
    scanner: Scanner<'a>,
    locals: Vec<Local<'a>>,
    scope_depth: usize,
}

impl<'a> Compiler<'a> {
    fn get_rule(kind: TokenKind) -> ParseRule<'a> {
        match kind {
            TokenKind::LeftParen => (Some(Self::grouping), None, Precedence::None),
            TokenKind::Minus => (Some(Self::unary), Some(Self::binary), Precedence::Term),
            TokenKind::Plus => (None, Some(Self::binary), Precedence::Term),
            TokenKind::Slash => (None, Some(Self::binary), Precedence::Factor),
            TokenKind::Star => (None, Some(Self::binary), Precedence::Factor),
            TokenKind::Bang => (Some(Self::unary), None, Precedence::None),
            TokenKind::BangEqual => (None, Some(Self::binary), Precedence::Equality),
            TokenKind::EqualEqual => (None, Some(Self::binary), Precedence::Equality),
            TokenKind::Greater => (None, Some(Self::binary), Precedence::Comparison),
            TokenKind::GreaterEqual => (None, Some(Self::binary), Precedence::Comparison),
            TokenKind::Less => (None, Some(Self::binary), Precedence::Comparison),
            TokenKind::LessEqual => (None, Some(Self::binary), Precedence::Comparison),
            TokenKind::Identifier => (Some(Self::variable), None, Precedence::None),
            TokenKind::String => (Some(Self::string), None, Precedence::None),
            TokenKind::Number => (Some(Self::number), None, Precedence::None),
            TokenKind::And => (None, Some(Self::and), Precedence::And),
            TokenKind::False => (Some(Self::literal), None, Precedence::None),
            TokenKind::True => (Some(Self::literal), None, Precedence::None),
            TokenKind::Nil => (Some(Self::literal), None, Precedence::None),
            TokenKind::Or => (None, Some(Self::or), Precedence::Or),
            _ => (None, None, Precedence::None),
        }
    }

    pub fn new(scanner: Scanner<'a>) -> Compiler<'a> {
        let function_name = "script";
        let function = Function {
            arity: 0,
            chunk: chunk::Handle::new(function_name),
            name: string::Handle::from_str(function_name),
        };
        Compiler {
            parser: Parser {
                previous: None,
                current: None,
                had_error: false,
                panic_mode: false,
            },
            scanner: scanner,
            current_function: function,
            function_type: FunctionType::Script,
            scope_depth: 0,
            locals: vec![Local {
                depth: Some(0),
                name: "",
            }],
        }
    }

    fn current_chunk_mut(&mut self) -> &mut Chunk {
        self.current_function.chunk.get_chunk_mut()
    }

    fn current_chunk(&mut self) -> &Chunk {
        self.current_function.chunk.get_chunk()
    }

    fn previous_kind(&self) -> TokenKind {
        self.parser.previous.as_ref().unwrap().kind
    }

    fn error_at(&mut self, location: ErrorLocation, message: &str) {
        if self.parser.panic_mode {
            return;
        }

        let token = match location {
            ErrorLocation::Current => self.parser.current.as_ref(),
            ErrorLocation::Previous => self.parser.previous.as_ref(),
            ErrorLocation::End => None,
        };

        let line = if let Some(token) = token {
            token.line
        } else {
            self.scanner.lines
        };

        eprint!("[line {}] Error", line);

        if token.is_none() {
            eprint!(" at end");
        } else if token.unwrap().kind != TokenKind::Error {
            eprint!(" at '{}'", token.unwrap().lexeme);
        }

        eprintln!(": {}", message);
        drop(token);
        self.parser.panic_mode = true;
        self.parser.had_error = true;
    }

    fn error_at_current(&mut self, message: &str) {
        if self.parser.current.is_none() {
            self.error_at(ErrorLocation::End, message)
        } else {
            self.error_at(ErrorLocation::Current, message)
        }
    }

    fn error(&mut self, message: &str) {
        self.error_at(ErrorLocation::Previous, message)
    }

    fn advance(&mut self) {
        self.parser.previous = std::mem::take(&mut self.parser.current);

        loop {
            self.parser.current = self.scanner.next();
            if self.parser.current.is_none() {
                break;
            }
            if self.parser.current.as_ref().unwrap().kind != TokenKind::Error {
                break;
            }

            self.error_at_current(&self.parser.current.as_ref().unwrap().lexeme);
        }
    }

    fn consume(&mut self, kind: TokenKind, message: &str) {
        if self.check(kind) {
            self.advance();
            return;
        }

        self.error_at_current(message);
    }

    fn check(&self, kind: TokenKind) -> bool {
        if let Some(token) = self.parser.current {
            token.kind == kind
        } else {
            false
        }
    }

    fn match_current(&mut self, kind: TokenKind) -> bool {
        if !self.check(kind) {
            return false;
        }

        self.advance();
        true
    }

    fn emit_byte(&mut self, byte: u8) {
        let line = self.parser.previous.as_ref().unwrap().line;
        self.current_chunk_mut().write(byte, line)
    }

    fn emit_op(&mut self, op: Op) {
        self.emit_byte(op as u8)
    }

    fn emit_bytes(&mut self, byte1: u8, byte2: u8) {
        self.emit_byte(byte1);
        self.emit_byte(byte2);
    }

    fn emit_loop(&mut self, loop_start: usize) {
        self.emit_op(Op::Loop);

        let offset: u16 = match (self.current_chunk_mut().code.len() - loop_start + 2).try_into() {
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
        return self.current_chunk_mut().code.len() - 2;
    }

    fn emit_return(&mut self) {
        self.emit_op(Op::Return);
    }

    fn make_constant(&mut self, value: Value) -> u8 {
        match self.current_chunk_mut().add_constant(value) {
            Ok(constant) => constant,
            Err(()) => {
                self.error("Too many constants in one chunk.");
                0
            }
        }
    }

    fn emit_constant(&mut self, value: Value) {
        let constant = self.make_constant(value);
        self.emit_bytes(Op::Constant as u8, constant);
    }

    fn patch_jump(&mut self, offset: usize) {
        let jump: u16 = match (self.current_chunk_mut().code.len() - offset - 2).try_into() {
            Ok(value) => value,
            Err(_) => {
                self.error("Too much code to jump over.");
                0
            }
        };

        self.current_chunk_mut().code[offset] = ((jump >> 8) & 0xff) as u8;
        self.current_chunk_mut().code[offset + 1] = (jump & 0xff) as u8;
    }

    fn binary(&mut self, _can_assign: bool) {
        let operator_type = self.previous_kind();
        let rule = Self::get_rule(operator_type);
        let precedence = rule.2.higher();
        self.parse_precedence(precedence);

        match operator_type {
            TokenKind::BangEqual => self.emit_bytes(Op::Equal as u8, Op::Not as u8),
            TokenKind::EqualEqual => self.emit_op(Op::Equal),
            TokenKind::Greater => self.emit_op(Op::Greater),
            TokenKind::GreaterEqual => self.emit_bytes(Op::Less as u8, Op::Not as u8),
            TokenKind::Less => self.emit_op(Op::Less),
            TokenKind::LessEqual => self.emit_bytes(Op::Greater as u8, Op::Not as u8),
            TokenKind::Plus => self.emit_op(Op::Add),
            TokenKind::Minus => self.emit_op(Op::Subtract),
            TokenKind::Star => self.emit_op(Op::Multiply),
            TokenKind::Slash => self.emit_op(Op::Divide),
            _ => unreachable!(),
        }
    }

    fn literal(&mut self, _can_assign: bool) {
        match self.previous_kind() {
            TokenKind::False => self.emit_op(Op::False),
            TokenKind::Nil => self.emit_op(Op::Nil),
            TokenKind::True => self.emit_op(Op::True),
            _ => (),
        }
    }

    fn grouping(&mut self, _can_assign: bool) {
        self.expression();
        self.consume(TokenKind::RightParen, "Expect ')' after expression.")
    }

    fn string(&mut self, _can_assign: bool) {
        let string = String::from(self.parser.previous.as_ref().unwrap().lexeme);

        self.emit_constant(Value::String(string::Handle::from_str(
            &string[1..string.len() - 1],
        )))
    }

    fn named_variable(&mut self, name: &str, can_assign: bool) {
        let get_op: Op;
        let set_op: Op;

        let arg = match self.resolve_local(name) {
            Some(arg) => {
                get_op = Op::GetLocal;
                set_op = Op::SetLocal;
                arg
            }
            _ => {
                get_op = Op::GetGlobal;
                set_op = Op::SetGlobal;
                self.identifier_constant(name)
            }
        };

        if can_assign && self.match_current(TokenKind::Equal) {
            self.expression();
            self.emit_bytes(set_op as u8, arg);
        } else {
            self.emit_bytes(get_op as u8, arg);
        }
    }

    fn variable(&mut self, can_assign: bool) {
        self.named_variable(self.parser.previous.as_ref().unwrap().lexeme, can_assign)
    }

    fn number(&mut self, _can_assign: bool) {
        let value: f64 = self
            .parser
            .previous
            .as_ref()
            .unwrap()
            .lexeme
            .parse()
            .expect("Failed to parse string into float");

        self.emit_constant(Value::Number(value));
    }

    fn or(&mut self, _can_assign: bool) {
        let else_jump = self.emit_jump(Op::JumpIfFalse);
        let end_jump = self.emit_jump(Op::Jump);

        self.patch_jump(else_jump);
        self.emit_op(Op::Pop);

        self.parse_precedence(Precedence::Or);
        self.patch_jump(end_jump);
    }

    fn unary(&mut self, _can_assign: bool) {
        let operator_type = self.previous_kind();

        self.parse_precedence(Precedence::Unary);

        match operator_type {
            TokenKind::Minus => self.emit_op(Op::Negate),
            TokenKind::Bang => self.emit_op(Op::Not),
            _ => unreachable!(),
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        let prefix_rule = Self::get_rule(self.previous_kind()).0;
        if prefix_rule.is_none() {
            self.error("Expect expression.");
            return;
        }

        let can_assign = precedence <= Precedence::Assignment;
        prefix_rule.unwrap()(self, can_assign);

        while self.parser.current.is_some()
            && precedence <= Self::get_rule(self.parser.current.as_ref().unwrap().kind).2
        {
            self.advance();
            if let Some(infix_rule) = Self::get_rule(self.previous_kind()).1 {
                infix_rule(self, can_assign);
            }
        }

        if can_assign && self.match_current(TokenKind::Equal) {
            self.error("Invalid assignment target.");
        }
    }

    fn identifier_constant(&mut self, name: &str) -> u8 {
        self.make_constant(Value::String(string::Handle::from_str(name)))
    }

    fn resolve_local(&mut self, name: &str) -> Option<u8> {
        for (index, local) in self.locals.iter().enumerate().rev() {
            if local.name == name {
                if local.depth.is_none() {
                    self.error("Can't read local variable in its own initializer.");
                }
                return Some(index as u8);
            }
        }

        None
    }

    fn add_local(&mut self, name: Token<'a>) {
        if self.locals.len() >= u8::MAX as usize {
            self.error("Too many local variables in function.");
            return;
        }

        self.locals.push(Local {
            name: name.lexeme,
            depth: None,
        })
    }

    fn declare_variable(&mut self) {
        if self.scope_depth == 0 {
            return;
        }

        let name = &self.parser.previous.unwrap();
        let mut unique = true;
        for local in self.locals.iter().rev() {
            if local.depth.is_some() && local.depth.unwrap() < self.scope_depth {
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

    fn parse_variable(&mut self, message: &str) -> u8 {
        self.consume(TokenKind::Identifier, message);

        self.declare_variable();
        if self.scope_depth > 0 {
            return 0;
        }

        return self.identifier_constant(self.parser.previous.as_ref().unwrap().lexeme);
    }

    fn mark_initialized(&mut self) {
        self.locals.last_mut().unwrap().depth = Some(self.scope_depth);
    }

    fn define_variable(&mut self, global: u8) {
        if self.scope_depth > 0 {
            self.mark_initialized();
            return;
        }

        self.emit_bytes(Op::DefineGlobal as u8, global)
    }

    fn and(&mut self, _can_assign: bool) {
        let end_jump = self.emit_jump(Op::JumpIfFalse);

        self.emit_op(Op::Pop);
        self.parse_precedence(Precedence::And);

        self.patch_jump(end_jump);
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn block(&mut self) {
        while self.parser.current.is_some() && !self.check(TokenKind::RightBrace) {
            self.declaration();
        }

        self.consume(TokenKind::RightBrace, "Expect '}' after block.")
    }

    fn var_declaration(&mut self) {
        let global = self.parse_variable("Expect variable name.");

        if self.match_current(TokenKind::Equal) {
            self.expression();
        } else {
            self.emit_op(Op::Nil);
        }

        self.consume(
            TokenKind::Semicolon,
            "Expect ';' after variable declaration.",
        );
        self.define_variable(global);
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.consume(TokenKind::Semicolon, "Expect ';' after expression.");
        self.emit_op(Op::Pop)
    }

    fn for_statement(&mut self) {
        self.begin_scope();
        self.consume(TokenKind::LeftParen, "Expect '(' after 'for'.");
        if self.match_current(TokenKind::Semicolon) {
        } else if self.match_current(TokenKind::Var) {
            self.var_declaration();
        } else {
            self.expression_statement();
        }

        let mut loop_start = self.current_chunk_mut().code.len();
        let mut exit_jump = None;

        if !self.match_current(TokenKind::Semicolon) {
            self.expression();
            self.consume(TokenKind::Semicolon, "Expect ';' after loop condition.");

            exit_jump = Some(self.emit_jump(Op::JumpIfFalse));
            self.emit_op(Op::Pop);
        }

        if !self.match_current(TokenKind::RightParen) {
            let body_jump = self.emit_jump(Op::Jump);
            let increment_start = self.current_chunk_mut().code.len();
            self.expression();
            self.emit_op(Op::Pop);
            self.consume(TokenKind::RightParen, "Expect ')' after for clauses.");

            self.emit_loop(loop_start);
            loop_start = increment_start;
            self.patch_jump(body_jump);
        }

        self.statement();
        self.emit_loop(loop_start);

        if let Some(offset) = exit_jump {
            self.patch_jump(offset);
            self.emit_op(Op::Pop);
        }

        self.end_scope();
    }

    fn if_statement(&mut self) {
        self.consume(TokenKind::LeftParen, "Expect '(' after 'if'.");
        self.expression();
        self.consume(TokenKind::RightParen, "Expect ')' after condition.");

        let then_jump = self.emit_jump(Op::JumpIfFalse);
        self.emit_op(Op::Pop);
        self.statement();

        let else_jump = self.emit_jump(Op::Jump);
        self.patch_jump(then_jump);
        self.emit_op(Op::Pop);

        if self.match_current(TokenKind::Else) {
            self.statement();
        }
        self.patch_jump(else_jump);
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(TokenKind::Semicolon, "Expect ';' after value.");
        self.emit_op(Op::Print)
    }

    fn while_statement(&mut self) {
        let loop_start = self.current_chunk_mut().code.len();
        self.consume(TokenKind::LeftParen, "Expect '(' after 'while'.");
        self.expression();
        self.consume(TokenKind::RightParen, "Expect ')' after condition.");

        let exit_jump = self.emit_jump(Op::JumpIfFalse);
        self.emit_op(Op::Pop);
        self.statement();
        self.emit_loop(loop_start);

        self.patch_jump(exit_jump);
        self.emit_op(Op::Pop);
    }

    fn synchronize(&mut self) {
        while self.parser.current.is_some() {
            if self.previous_kind() == TokenKind::Semicolon {
                return;
            }
            match self.parser.current.as_ref().unwrap().kind {
                TokenKind::Fun
                | TokenKind::Var
                | TokenKind::For
                | TokenKind::If
                | TokenKind::While
                | TokenKind::Print
                | TokenKind::Return => return,
                _ => self.advance(),
            }
        }
    }

    fn declaration(&mut self) {
        if self.match_current(TokenKind::Var) {
            self.var_declaration();
        } else {
            self.statement();
        }

        if self.parser.panic_mode {
            self.synchronize();
        }
    }

    fn statement(&mut self) {
        if self.match_current(TokenKind::Print) {
            self.print_statement();
        } else if self.match_current(TokenKind::For) {
            self.for_statement();
        } else if self.match_current(TokenKind::If) {
            self.if_statement();
        } else if self.match_current(TokenKind::While) {
            self.while_statement();
        } else if self.match_current(TokenKind::LeftBrace) {
            self.begin_scope();
            self.block();
            self.end_scope();
        } else {
            self.expression_statement();
        }
    }

    fn end_compiler(&mut self) -> &Function {
        self.emit_return();
        {
            #![cfg(feature = "trace-execution")]
            let name = &self.current_function.name.as_str().string;
            if !self.parser.had_error {
                self.current_chunk().disassemble(name);
            }
        }
        &self.current_function
    }

    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn end_scope(&mut self) {
        self.scope_depth -= 1;

        while let Some(local) = self.locals.last() {
            if local.depth.unwrap() > self.scope_depth {
                self.emit_op(Op::Pop);
                self.locals.pop();
            } else {
                break;
            }
        }
    }

    fn compile(&mut self) -> Result<Function, InterpretError> {
        self.advance();

        while self.parser.current.is_some() {
            self.declaration();
        }

        let had_error = self.parser.had_error;
        let function = self.end_compiler();
        if had_error {
            Err(InterpretError::CompileError)
        } else {
            Ok(*function)
        }
    }
}

pub fn compile(source: &String) -> Result<Function, InterpretError> {
    let mut compiler = Compiler::new(Scanner::new(source));
    compiler.compile()
}
