use std::iter::{Enumerate, Peekable};

#[derive(Copy, Clone, PartialEq)]
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

#[derive(Copy, Clone)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub line: i32,
    pub lexeme: &'a str,
}

pub struct Scanner<'a> {
    source: &'a String,
    pub lines: i32,
    start: usize,
    iter: Peekable<Enumerate<std::str::Chars<'a>>>,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a String) -> Scanner<'a> {
        Scanner {
            source,
            lines: 1,
            start: 0,
            iter: source.chars().enumerate().peekable(),
        }
    }

    fn advance(&mut self) -> Option<(usize, char)> {
        self.iter.next()
    }

    fn consume_while(&mut self, fun: fn(c: char) -> bool) {
        while self.iter.next_if(|&(_, c)| fun(c)).is_some() {}
    }

    fn match_current(&mut self, expected: char) -> bool {
        self.iter.next_if(|&(_, c)| c == expected).is_some()
    }

    fn skip_whitespace(&mut self) {
        while let Some((_, c)) = self.iter.peek() {
            match *c {
                ' ' | '\r' | '\t' => {
                    self.advance();
                }
                '\n' => {
                    self.lines += 1;
                    self.advance();
                }
                '/' => {
                    if let Some((_, '/')) = self.peek_next() {
                        self.consume_while(|c| c != '\n');
                    } else {
                        return;
                    }
                }
                _ => return,
            }
        }
    }

    fn get_lexeme(&mut self) -> &'a str {
        let end = if let Some((number, _)) = self.iter.peek() {
            *number
        } else {
            self.source.len()
        };

        &self.source[self.start..end]
    }

    fn make_token(&mut self, kind: TokenKind) -> Token<'a> {
        Token {
            kind: kind,
            line: self.lines,
            lexeme: self.get_lexeme(),
        }
    }

    fn make_error_token(&self, message: &'static str) -> Token<'a> {
        Token {
            kind: TokenKind::Error,
            line: self.lines,
            lexeme: message,
        }
    }

    fn peek_next(&mut self) -> Option<(usize, char)> {
        if let Some((n, _)) = self.iter.peek() {
            if let Some(byte) = self.source.as_bytes().get(n + 1) {
                return Some((n + 1, *byte as char));
            }
        }

        None
    }

    fn string(&mut self) -> Token<'a> {
        while let Some((_, c)) = self.iter.next_if(|&(_, c)| c != '"') {
            if c == '\n' {
                self.lines += 1;
            }
        }

        if self.match_current('"') {
            self.make_token(TokenKind::String)
        } else {
            self.make_error_token("Unterminated string.")
        }
    }

    fn number(&mut self) -> Token<'a> {
        self.consume_while(|c| c.is_digit(10));

        // Look for a fractional part.
        if let Some((_, '.')) = self.iter.peek() {
            if let Some((_, '0'..='9')) = self.peek_next() {
                // Consume the ".".
                self.advance();
                self.consume_while(|c| c.is_digit(10));
            }
        }

        self.make_token(TokenKind::Number)
    }

    fn identifier(&mut self) -> Token<'a> {
        self.consume_while(|c| c.is_ascii_alphanumeric() || c == '_');

        let lexeme = self.get_lexeme();
        let kind = match lexeme {
            "and" => TokenKind::And,
            "class" => TokenKind::Class,
            "else" => TokenKind::Else,
            "false" => TokenKind::False,
            "for" => TokenKind::For,
            "fun" => TokenKind::Fun,
            "if" => TokenKind::If,
            "nil" => TokenKind::Nil,
            "or" => TokenKind::Or,
            "print" => TokenKind::Print,
            "return" => TokenKind::Return,
            "super" => TokenKind::Super,
            "this" => TokenKind::This,
            "true" => TokenKind::True,
            "var" => TokenKind::Var,
            "while" => TokenKind::While,
            _ => TokenKind::Identifier,
        };

        Token {
            kind: kind,
            lexeme: lexeme,
            line: self.lines,
        }
    }
}

impl<'a> Iterator for Scanner<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Token<'a>> {
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
            '/' => self.make_token(TokenKind::Slash),
            '!' => {
                if self.match_current('=') {
                    self.make_token(TokenKind::BangEqual)
                } else {
                    self.make_token(TokenKind::Bang)
                }
            }
            '=' => {
                if self.match_current('=') {
                    self.make_token(TokenKind::EqualEqual)
                } else {
                    self.make_token(TokenKind::Equal)
                }
            }
            '<' => {
                if self.match_current('=') {
                    self.make_token(TokenKind::LessEqual)
                } else {
                    self.make_token(TokenKind::Less)
                }
            }
            '>' => {
                if self.match_current('=') {
                    self.make_token(TokenKind::GreaterEqual)
                } else {
                    self.make_token(TokenKind::Greater)
                }
            }
            '"' => self.string(),
            '0'..='9' => self.number(),
            'a'..='z' | 'A'..='Z' | '_' => self.identifier(),
            _ => self.make_error_token("Unexpected character."),
        };

        Some(token)
    }
}
