use std::iter::{Enumerate, Peekable};

#[repr(u8)]
pub enum TokenKind {
  // Single-character tokens.
  LeftParen,
  RightParen,
  LeftBrace,
  RightBrace,
  Comma,
  Dot,
  Minus,
  Plus,
  Semicolon,
  Slash,
  Star,
  // One or two character tokens.
  Bang,
  BangEqual,
  Equal,
  EqualEqual,
  Greater,
  GreaterEqual,
  Less,
  LessEqual,
  // Literals.
  Identifier,
  String,
  Number,
  // Keywords.
  And,
  Class,
  Else,
  False,
  For,
  Fun,
  If,
  Nil,
  Or,
  Print,
  Return,
  Super,
  This,
  True,
  Var,
  While,
  Error,
}

pub struct Token<'a> {
  pub kind: TokenKind,
  pub line: i32,
  pub lexeme: &'a str,
}

pub struct Scanner<'a> {
  source: &'a String,
  lines: i32,
  start: usize,
  iter: Peekable<Enumerate<std::str::Chars<'a>>>,
}

impl<'a> Scanner<'a> {
  pub fn new(source: &'a String) -> Scanner<'a> {
    Scanner {
      source: source,
      lines: 0,
      start: 0,
      iter: source.chars().enumerate().peekable(),
    }
  }

  fn advance(&mut self) -> Option<(usize, char)> {
    self.iter.next()
  }

  fn match_next(&mut self, expected: char) -> bool {
    match self.iter.next_if(|&(_, c)| c == expected) {
      Some(_) => true,
      None => false,
    }
  }

  fn skip_whitespace(&mut self) {
    while let Some((_, c)) = self.iter.next_if(|&(_, c)| c.is_whitespace()) {
      if c == '\n' {
        self.lines += 1;
      }
    }
  }

  fn make_token(&mut self, kind: TokenKind) -> Token<'a> {
    let end = if let Some((number, _)) = self.iter.peek() {
      *number
    } else {
      self.start + 1
    };
    Token {
      kind: kind,
      line: self.lines,
      lexeme: &self.source[self.start..end],
    }
  }

  fn make_error_token(&self, message: &'static str) -> Token<'a> {
    Token {
      kind: TokenKind::Error,
      line: self.lines,
      lexeme: message,
    }
  }

  pub fn scan_token(&mut self) -> Option<Token<'a>> {
    self.skip_whitespace();

    let next = self.advance();

    if next.is_none() {
      return None;
    }

    let (start, c) = next.unwrap();
    self.start = start;

    let token = match c {
      '(' => self.make_token(TokenKind::LeftParen),
      ')' => self.make_token(TokenKind::RightParen),
      '{' => self.make_token(TokenKind::LeftBrace),
      '}' => self.make_token(TokenKind::RightBrace),
      ';' => self.make_token(TokenKind::Semicolon),
      ',' => self.make_token(TokenKind::Comma),
      '.' => self.make_token(TokenKind::Dot),
      '-' => self.make_token(TokenKind::Minus),
      '+' => self.make_token(TokenKind::Plus),
      '*' => self.make_token(TokenKind::Star),
      '/' => {
        // handle comments here
        if self.match_next('/') {
          while self.iter.next_if(|&(_, c)| c != '\n').is_some() {}
          return self.scan_token();
        }

        self.make_token(TokenKind::Slash)
      }
      '!' => {
        if self.match_next('=') {
          self.make_token(TokenKind::BangEqual)
        } else {
          self.make_token(TokenKind::Bang)
        }
      }
      '=' => {
        if self.match_next('=') {
          self.make_token(TokenKind::EqualEqual)
        } else {
          self.make_token(TokenKind::Equal)
        }
      }
      '<' => {
        if self.match_next('=') {
          self.make_token(TokenKind::LessEqual)
        } else {
          self.make_token(TokenKind::Less)
        }
      }
      '>' => {
        if self.match_next('=') {
          self.make_token(TokenKind::GreaterEqual)
        } else {
          self.make_token(TokenKind::Greater)
        }
      }
      _ => self.make_error_token("Unexpected character."),
    };

    Some(token)
  }
}
