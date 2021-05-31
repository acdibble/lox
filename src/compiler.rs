use crate::chunk::*;
use crate::scanner::*;
use crate::value::*;

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

type ParseFn<'a> = fn(&mut Compiler<'a>);
type ParseRule<'a> = (Option<ParseFn<'a>>, Option<ParseFn<'a>>, Precedence);

struct Parser<'a> {
  previous: Option<Token<'a>>,
  current: Option<Token<'a>>,
  had_error: bool,
  panic_mode: bool,
}

pub struct Compiler<'a> {
  parser: Parser<'a>,
  scanner: Scanner<'a>,
  chunk: &'a mut Chunk,
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
      TokenKind::Number => (Some(Self::number), None, Precedence::None),
      TokenKind::False => (Some(Self::literal), None, Precedence::None),
      TokenKind::True => (Some(Self::literal), None, Precedence::None),
      TokenKind::Nil => (Some(Self::literal), None, Precedence::None),
      _ => (None, None, Precedence::None),
    }
  }

  pub fn new(scanner: Scanner<'a>, chunk: &'a mut Chunk) -> Compiler<'a> {
    Compiler {
      parser: Parser {
        previous: None,
        current: None,
        had_error: false,
        panic_mode: false,
      },
      scanner: scanner,
      chunk: chunk,
    }
  }

  fn current_kind(&self) -> TokenKind {
    self.parser.current.as_ref().unwrap().kind
  }

  fn previous_kind(&self) -> TokenKind {
    self.parser.previous.as_ref().unwrap().kind
  }

  fn error_at(&mut self, line: i32, kind: TokenKind, message: &'a str) {
    if self.parser.panic_mode {
      return;
    }
    self.parser.panic_mode = true;
    eprint!("[line {}] Error", line);

    if kind == TokenKind::EOF {
      eprint!(" at end");
    } else if kind != TokenKind::Error {
      eprint!(" at '{}'", message);
    }

    eprintln!(": {}", message);
    self.parser.had_error = true;
  }

  fn error_at_current(&mut self, message: &'a str) {
    self.error_at(
      self.parser.current.as_ref().unwrap().line,
      self.current_kind(),
      message,
    )
  }

  fn error(&mut self, message: &'a str) {
    self.error_at(
      self.parser.previous.as_ref().unwrap().line,
      self.previous_kind(),
      message,
    )
  }

  fn advance(&mut self) {
    self.parser.previous = std::mem::take(&mut self.parser.current);

    loop {
      let token = self.scanner.scan_token();
      self.parser.current = Some(token);
      let token = self.parser.current.as_ref().unwrap();
      if token.kind != TokenKind::Error {
        break;
      }

      self.error_at_current(&self.parser.current.as_ref().unwrap().lexeme);
    }
  }

  fn consume(&mut self, kind: TokenKind, message: &'static str) {
    if self.current_kind() == kind {
      self.advance();
      return;
    }

    self.error_at_current(message);
  }

  fn emit_byte(&mut self, byte: u8) {
    self
      .chunk
      .write(byte, self.parser.previous.as_ref().unwrap().line);
  }

  fn emit_bytes(&mut self, byte1: u8, byte2: u8) {
    self.emit_byte(byte1);
    self.emit_byte(byte2);
  }

  fn emit_return(&mut self) {
    self.emit_byte(Op::Return as u8);
  }

  fn make_constant(&mut self, value: Value) -> u8 {
    let constant = self.chunk.add_constant(value);
    if constant > std::u8::MAX {
      self.error("Too many constants in one chunk.");
      0
    } else {
      constant as u8
    }
  }

  fn emit_constant(&mut self, value: Value) {
    let constant = self.make_constant(value);
    self.emit_bytes(Op::Constant as u8, constant);
  }

  fn binary(&mut self) {
    let operator_type = self.previous_kind();
    let rule = Self::get_rule(operator_type);
    let precedence = rule.2.higher();
    self.parse_precedence(precedence);

    match operator_type {
      TokenKind::Plus => self.emit_byte(Op::Add as u8),
      TokenKind::Minus => self.emit_byte(Op::Subtract as u8),
      TokenKind::Star => self.emit_byte(Op::Multiply as u8),
      TokenKind::Slash => self.emit_byte(Op::Divide as u8),
      _ => unreachable!(),
    }
  }

  fn literal(&mut self) {
    match self.previous_kind() {
      TokenKind::False => self.emit_byte(Op::False as u8),
      TokenKind::Nil => self.emit_byte(Op::Nil as u8),
      TokenKind::True => self.emit_byte(Op::True as u8),
      _ => (),
    }
  }

  fn grouping(&mut self) {
    self.expression();
    self.consume(TokenKind::RightParen, "Expect ')' after expression.")
  }

  fn number(&mut self) {
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

  fn unary(&mut self) {
    let operator_type = self.previous_kind();

    self.parse_precedence(Precedence::Unary);

    match operator_type {
      TokenKind::Minus => self.emit_byte(Op::Negate as u8),
      TokenKind::Bang => self.emit_byte(Op::Not as u8),
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

    prefix_rule.unwrap()(self);

    while precedence <= Self::get_rule(self.current_kind()).2 {
      self.advance();
      if let Some(infix_rule) = Self::get_rule(self.previous_kind()).1 {
        infix_rule(self);
      }
    }
  }

  fn expression(&mut self) {
    self.parse_precedence(Precedence::Assignment);
  }

  fn end_compiler(&mut self) {
    self.emit_return();
    if !self.parser.had_error {
      self.chunk.disassemble("code");
    }
  }

  fn compile(&mut self) -> bool {
    self.advance();
    self.expression();
    self.end_compiler();
    !self.parser.had_error
  }
}

pub fn compile(source: &String, chunk: &mut Chunk) -> bool {
  let scanner = Scanner::new(source);
  let mut compiler = Compiler::new(scanner, chunk);
  compiler.compile()
}
