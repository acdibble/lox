const enum TokenType {
  // Single-character tokens.
  LeftParen,
  RightParen,
  LeftBrace,
  RightBrace,
  Colon,
  Comma,
  Dot,
  Minus,
  Plus,
  Semicolon,
  Slash,
  Star,
  QuestionMark,

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
  Break,
  Class,
  Else,
  False,
  Fun,
  For,
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

  EOF,
}

export default TokenType;
