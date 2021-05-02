import {
  Binary,
  Comma,
  Expr,
  Grouping,
  Literal,
  Ternary,
  Unary,
} from './Expr.js';
import type { LoxError } from './main.js';
import Token from './Token.js';
import TokenType from './TokenType.js';

class ParserError extends Error {
  constructor() {
    super();

    if (!this.stack) {
      Error.captureStackTrace(this, ParserError);
    }
  }
}

export default class Parser {
  private readonly discardFunctions = {
    [TokenType.BangEqual]: this.comparison,
    [TokenType.EqualEqual]: this.comparison,
    [TokenType.Greater]: this.term,
    [TokenType.GreaterEqual]: this.term,
    [TokenType.Less]: this.term,
    [TokenType.LessEqual]: this.term,
    [TokenType.Minus]: this.factor,
    [TokenType.Plus]: this.factor,
    [TokenType.Slash]: this.unary,
    [TokenType.Star]: this.unary,
  } as const;

  private current = 0;

  constructor(
    private readonly tokens: readonly Token[],
    private readonly loxError: LoxError,
  ) {}

  parse(): Expr | null {
    try {
      return this.expression();
    } catch (error) {
      if (error instanceof ParserError) return null;
      throw error;
    }
  }

  private expression(): Expr {
    const expr = this.equality();

    if (this.match(TokenType.QuestionMark)) {
      const exprIfTrue = this.expression();
      this.consume(TokenType.Colon, "Expect ':' after expression");
      const exprIfFalse = this.expression();
      return new Ternary(expr, exprIfTrue, exprIfFalse);
    }

    return expr;
  }

  private equality(): Expr {
    let expr = this.comparison();

    while (this.match(TokenType.BangEqual, TokenType.EqualEqual)) {
      const operator = this.previous();
      const right = this.comparison();
      expr = new Binary(expr, operator, right);
    }

    return expr;
  }

  private comparison(): Expr {
    let expr = this.term();

    while (this.match(TokenType.Greater, TokenType.GreaterEqual, TokenType.Less, TokenType.LessEqual)) {
      const operator = this.previous();
      const right = this.term();
      expr = new Binary(expr, operator, right);
    }

    return expr;
  }

  private term(): Expr {
    let expr = this.factor();

    while (this.match(TokenType.Minus, TokenType.Plus)) {
      const operator = this.previous();
      const right = this.factor();
      expr = new Binary(expr, operator, right);
    }

    return expr;
  }

  private factor(): Expr {
    let expr = this.unary();

    while (this.match(TokenType.Slash, TokenType.Star)) {
      const operator = this.previous();
      const right = this.unary();
      expr = new Binary(expr, operator, right);
    }

    return expr;
  }

  private unary(): Expr {
    if (this.match(TokenType.Bang, TokenType.Minus)) {
      const operator = this.previous();
      const right = this.unary();
      return new Unary(operator, right);
    }

    return this.primary();
  }

  private primary(): Expr {
    if (this.match(TokenType.False)) return new Literal(false);
    if (this.match(TokenType.True)) return new Literal(true);
    if (this.match(TokenType.Nil)) return new Literal(null);

    if (this.match(TokenType.Number, TokenType.String)) {
      return new Literal(this.previous().literal);
    }

    if (this.match(TokenType.LeftParen)) {
      const exprs = [this.expression()];
      while (this.match(TokenType.Comma)) exprs.push(this.expression());
      this.consume(TokenType.RightParen, "Expect ')' after expression.");
      if (exprs.length === 1) return new Grouping(exprs[0]!);
      return new Comma(exprs);
    }

    if (this.handleMalformedBinaryExpression()) {
      return this.expression();
    }

    throw this.error(this.peek(), 'Expect expression.');
  }

  private match(...types: TokenType[]): boolean {
    for (const type of types) {
      if (this.check(type)) {
        this.advance();
        return true;
      }
    }

    return false;
  }

  private consume(type: TokenType, message: string): Token {
    if (this.check(type)) return this.advance();

    throw this.error(this.peek(), message);
  }

  private check(type: TokenType): boolean {
    if (this.isAtEnd()) return false;
    return this.peek().type === type;
  }

  private advance(): Token {
    if (!this.isAtEnd()) this.current++;
    return this.previous();
  }

  private isAtEnd(): boolean {
    return this.peek().type === TokenType.EOF;
  }

  private peek(): Token {
    return this.tokens[this.current]!;
  }

  private previous(): Token {
    return this.tokens[this.current - 1]!;
  }

  private error(token: Token, message: string): ParserError {
    this.loxError(token, message);
    return new ParserError();
  }

  private synchronize(): void {
    this.advance();

    while (!this.isAtEnd()) {
      if (this.previous().type === TokenType.Semicolon) return;

      // eslint-disable-next-line default-case
      switch (this.peek().type) {
        case TokenType.Class:
        case TokenType.Fun:
        case TokenType.Var:
        case TokenType.For:
        case TokenType.If:
        case TokenType.While:
        case TokenType.Print:
        case TokenType.Return:
          return;
      }

      this.advance();
    }
  }

  private handleMalformedBinaryExpression(): boolean {
    // eslint-disable-next-line prefer-spread
    if (this.match.apply(this, Object.keys(this.discardFunctions) as TokenType[])) {
      const operator = this.previous();
      this.discardFunctions[operator.type as keyof typeof Parser.prototype.discardFunctions].call(this);
      this.error(operator, `Expect let hand operand for ${operator.lexeme}`);
      return true;
    }

    return false;
  }
}
