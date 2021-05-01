import type { LoxError } from './main.js';
import Token from './Token.js';
import TokenType from './TokenType.js';

export default class Scanner {
  static keywords: Record<string, TokenType> = {
    and: TokenType.And,
    class: TokenType.Class,
    else: TokenType.Else,
    false: TokenType.False,
    for: TokenType.For,
    fun: TokenType.Fun,
    if: TokenType.If,
    nil: TokenType.Nil,
    or: TokenType.Or,
    print: TokenType.Print,
    return: TokenType.Return,
    super: TokenType.Super,
    this: TokenType.This,
    true: TokenType.True,
    var: TokenType.Var,
    while: TokenType.While,
  };

  private static isDigit(char: string): boolean {
    return char >= '0' && char <= '9';
  }

  private static isAlpha(char: string): boolean {
    return (char >= 'a' && char <= 'z') || (char >= 'A' && char <= 'Z') || char === '_';
  }

  private static isAlphaNumeric(char: string): boolean {
    return this.isDigit(char) || this.isAlpha(char);
  }

  private readonly tokens: Token[] = [];
  private start = 0;
  private current = 0;
  private line = 1;

  constructor(
    private readonly source: string,
    private readonly error: LoxError,
  ) {}

  private advance(): string {
    return this.source[this.current++]!;
  }

  private addToken(type: TokenType): void
  private addToken(type: TokenType.String, literal: string): void
  private addToken(type: TokenType.Number, literal: number): void
  private addToken(type: TokenType, literal: any = null): void {
    const text = this.source.slice(this.start, this.current);
    this.tokens.push(new Token(type, text, literal, this.line));
  }

  private match(expected: string): boolean {
    if (this.isAtEnd()) return false;
    if (this.source[this.current] !== expected) return false;

    this.current++;
    return true;
  }

  private peek(): string {
    return this.source[this.current] ?? '\0';
  }

  private peekNext(): string {
    return this.source[this.current + 1] ?? '\0';
  }

  private string(): void {
    while (this.peek() !== '"' && !this.isAtEnd()) {
      if (this.peek() === '\n') this.line++;
      this.advance();
    }

    if (this.isAtEnd()) {
      this.error(this.line, 'Unterminated string.');
      return;
    }

    // consume second "
    this.advance();

    const value = this.source.slice(this.start + 1, this.current - 1);
    this.addToken(TokenType.String, value);
  }

  private number(): void {
    while (Scanner.isDigit(this.peek())) {
      this.advance();
    }

    if (this.peek() === '.' && Scanner.isDigit(this.peekNext())) {
      // consume the .
      this.advance();

      while (Scanner.isDigit(this.peek())) {
        this.advance();
      }
    }

    this.addToken(TokenType.Number, Number.parseFloat(this.source.slice(this.start, this.current)));
  }

  private identifier(): void {
    while (Scanner.isAlphaNumeric(this.peek())) {
      this.advance();
    }

    const text = this.source.slice(this.start, this.current);
    const type = Scanner.keywords[text] ?? TokenType.Identifier;
    this.addToken(type);
  }

  private scanToken(): void {
    const char = this.advance();
    switch (char) {
      case '(':
        this.addToken(TokenType.LeftParen);
        break;
      case ')':
        this.addToken(TokenType.RightParen);
        break;
      case '{':
        this.addToken(TokenType.LeftBrace);
        break;
      case '}':
        this.addToken(TokenType.RightBrace);
        break;
      case ',':
        this.addToken(TokenType.Comma);
        break;
      case '.':
        this.addToken(TokenType.Dot);
        break;
      case '-':
        this.addToken(TokenType.Minus);
        break;
      case '+':
        this.addToken(TokenType.Plus);
        break;
      case ';':
        this.addToken(TokenType.Semicolon);
        break;
      case '*':
        this.addToken(TokenType.Star);
        break;
      case '!':
        this.addToken(this.match('=') ? TokenType.BangEqual : TokenType.Bang);
        break;
      case '=':
        this.addToken(this.match('=') ? TokenType.EqualEqual : TokenType.Equal);
        break;
      case '<':
        this.addToken(this.match('=') ? TokenType.LessEqual : TokenType.Less);
        break;
      case '>':
        this.addToken(this.match('=') ? TokenType.GreaterEqual : TokenType.Greater);
        break;
      case '/':
        if (this.match('/')) {
          while (this.peek() !== '\n' && !this.isAtEnd()) this.advance();
        } else {
          this.addToken(TokenType.Slash);
        }
        break;
      case ' ':
      case '\r':
      case '\t':
        break;
      case '\n':
        this.line++;
        break;
      case '"':
        this.string();
        break;
      default:
        if (Scanner.isDigit(char)) {
          this.number();
        } else if (Scanner.isAlpha(char)) {
          this.identifier();
        } else {
          this.error(this.line, 'Unexpected character.');
        }
        break;
    }
  }

  private isAtEnd(): boolean {
    return this.current >= this.source.length;
  }

  scanTokens(): Token[] {
    while (!this.isAtEnd()) {
      this.start = this.current;
      this.scanToken();
    }

    this.tokens.push(new Token(TokenType.EOF, '', null, this.line));
    return this.tokens;
  }

  [Symbol.iterator](): IterableIterator<Token> {
    return this.scanTokens()[Symbol.iterator]();
  }
}
