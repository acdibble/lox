import type { LoxError } from "./main.ts";
import Token from "./Token.ts";
import TokenType from "./TokenType.ts";

export default class Scanner {
  static keywords: Record<string, TokenType> = Object.assign(
    Object.create(null),
    {
      and: TokenType.And,
      break: TokenType.Break,
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
    },
  );

  private static isDigit(char: string): boolean {
    return char >= "0" && char <= "9";
  }

  private static isAlpha(char: string): boolean {
    return (char >= "a" && char <= "z") || (char >= "A" && char <= "Z") ||
      char === "_";
  }

  private static isAlphaNumeric(char: string): boolean {
    return this.isDigit(char) || this.isAlpha(char);
  }

  private start = 0;
  private current = 0;
  private line = 1;

  constructor(
    private readonly source: string,
    private readonly loxError: LoxError,
  ) {}

  private advance(): string {
    return this.source[this.current++]!;
  }

  private createToken(type: TokenType): Token;
  private createToken(type: TokenType.String, literal: string): Token;
  private createToken(type: TokenType.Number, literal: number): Token;
  private createToken(type: TokenType, literal: any = null): Token {
    const text = this.source.slice(this.start, this.current);
    return new Token(type, text, literal, this.line);
  }

  private match(expected: string): boolean {
    if (this.isAtEnd()) return false;
    if (this.source[this.current] !== expected) return false;

    this.current++;
    return true;
  }

  private peek(): string {
    return this.source[this.current] ?? "\0";
  }

  private peekNext(): string {
    return this.source[this.current + 1] ?? "\0";
  }

  private string(): Token | null {
    while (this.peek() !== '"' && !this.isAtEnd()) {
      if (this.peek() === "\n") this.line++;
      this.advance();
    }

    if (this.isAtEnd()) {
      this.loxError(this.line, "Unterminated string.");
      return null;
    }

    // consume second "
    this.advance();

    const value = this.source.slice(this.start + 1, this.current - 1);
    return this.createToken(TokenType.String, value);
  }

  private number(): Token {
    while (Scanner.isDigit(this.peek())) {
      this.advance();
    }

    if (this.peek() === "." && Scanner.isDigit(this.peekNext())) {
      // consume the .
      this.advance();

      while (Scanner.isDigit(this.peek())) {
        this.advance();
      }
    }

    return this.createToken(
      TokenType.Number,
      Number.parseFloat(this.source.slice(this.start, this.current)),
    );
  }

  private identifier(): Token {
    while (Scanner.isAlphaNumeric(this.peek())) {
      this.advance();
    }

    const text = this.source.slice(this.start, this.current);
    const type = Scanner.keywords[text] ?? TokenType.Identifier;
    return this.createToken(type);
  }

  private scanToken(): Token | null {
    const char = this.advance();
    switch (char) {
      case "(":
        return this.createToken(TokenType.LeftParen);
      case ")":
        return this.createToken(TokenType.RightParen);
      case "{":
        return this.createToken(TokenType.LeftBrace);
      case "}":
        return this.createToken(TokenType.RightBrace);
      case ",":
        return this.createToken(TokenType.Comma);
      case ".":
        return this.createToken(TokenType.Dot);
      case "-":
        return this.createToken(TokenType.Minus);
      case "+":
        return this.createToken(TokenType.Plus);
      case ";":
        return this.createToken(TokenType.Semicolon);
      case "*":
        return this.createToken(TokenType.Star);
      case "?":
        return this.createToken(TokenType.QuestionMark);
      case ":":
        return this.createToken(TokenType.Colon);
      case "!":
        return this.createToken(
          this.match("=") ? TokenType.BangEqual : TokenType.Bang,
        );
      case "=":
        return this.createToken(
          this.match("=") ? TokenType.EqualEqual : TokenType.Equal,
        );
      case "<":
        return this.createToken(
          this.match("=") ? TokenType.LessEqual : TokenType.Less,
        );
      case ">":
        return this.createToken(
          this.match("=") ? TokenType.GreaterEqual : TokenType.Greater,
        );
      case "/":
        if (this.match("/")) {
          while (this.peek() !== "\n" && !this.isAtEnd()) this.advance();
        } else if (this.match("*")) {
          while (this.peek() !== "*" && !this.match("/") && !this.isAtEnd()) {
            const current = this.advance();
            if (current === "\n") this.line++;
          }
          this.advance();
          this.advance();
        } else {
          return this.createToken(TokenType.Slash);
        }
        break;
      case " ":
      case "\r":
      case "\t":
        break;
      case "\n":
        this.line++;
        break;
      case '"':
        return this.string();
      default:
        if (Scanner.isDigit(char)) return this.number();
        if (Scanner.isAlpha(char)) return this.identifier();
        this.loxError(this.line, "Unexpected character.");
    }
    return null;
  }

  private isAtEnd(): boolean {
    return this.current >= this.source.length;
  }

  *[Symbol.iterator](): IterableIterator<Token> {
    while (!this.isAtEnd()) {
      this.start = this.current;
      const token = this.scanToken();
      if (token) yield token;
    }

    yield new Token(TokenType.EOF, "", null, this.line);
  }
}
