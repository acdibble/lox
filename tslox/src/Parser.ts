import Expr from "./Expr.ts";
import type { LoxError } from "./main.ts";
import Stmt from "./Stmt.ts";
import Token from "./Token.ts";
import TokenType from "./TokenType.ts";

class ParseError extends Error {
  constructor() {
    super();

    if (!this.stack) {
      Error.captureStackTrace(this, ParseError);
    }
  }
}

export default class Parser {
  private readonly discardFunctions = new Map([
    [TokenType.BangEqual, this.comparison],
    [TokenType.EqualEqual, this.comparison],
    [TokenType.Greater, this.term],
    [TokenType.GreaterEqual, this.term],
    [TokenType.Less, this.term],
    [TokenType.LessEqual, this.term],
    [TokenType.Minus, this.factor],
    [TokenType.Plus, this.factor],
    [TokenType.Slash, this.unary],
    [TokenType.Star, this.unary],
  ]);

  private current = 0;

  constructor(
    private readonly tokens: readonly Token[],
    private readonly loxError: LoxError,
  ) {}

  parse(): Stmt[] {
    const statements: Stmt[] = [];
    while (!this.isAtEnd()) {
      const stmt = this.declaration();
      if (stmt) statements.push(stmt);
    }
    return statements;
  }

  private expression(): Expr {
    return this.assignment();
  }

  private declaration(): Stmt | null {
    try {
      if (this.match(TokenType.Class)) return this.classDeclaration();
      if (this.match(TokenType.Fun)) return this.function("function");
      if (this.match(TokenType.Var)) return this.varDeclaration();

      return this.statement();
    } catch (error) {
      if (error instanceof ParseError) {
        this.synchronize();
        return null;
      }
      throw error;
    }
  }

  private statement(): Stmt {
    if (this.match(TokenType.Break)) return this.breakStatement();
    if (this.match(TokenType.For)) return this.forStatement();
    if (this.match(TokenType.If)) return this.ifStatement();
    if (this.match(TokenType.Print)) return this.printStatement();
    if (this.match(TokenType.Return)) return this.returnStatement();
    if (this.match(TokenType.While)) return this.whileStatement();
    if (this.match(TokenType.LeftBrace)) return new Stmt.Block(this.block());
    return this.expressionStatement();
  }

  private classDeclaration(): Stmt {
    const name = this.consume(TokenType.Identifier, "Expect class name.");
    this.consume(TokenType.LeftBrace, "Expect '{' before class body.");

    const methods: Stmt.Function[] = [];
    const classMethods: Stmt.Function[] = [];
    while (!this.check(TokenType.RightBrace) && !this.isAtEnd()) {
      (this.match(TokenType.Class) ? classMethods : methods)
        .push(this.function("method"));
    }

    this.consume(TokenType.RightBrace, "Expect '}' after class body.");
    return new Stmt.Class(name, methods, classMethods);
  }

  private varDeclaration(): Stmt {
    const name = this.consume(TokenType.Identifier, "Expect variable name.");

    let initializer = null;
    if (this.match(TokenType.Equal)) initializer = this.expression();

    if (initializer instanceof Expr.Function) {
      initializer = new Expr.Function(
        name,
        initializer.params,
        initializer.body,
      );
    }

    this.consume(TokenType.Semicolon, "Expect ';' after variable declaration.");
    return new Stmt.Var(name, initializer);
  }

  private breakStatement(): Stmt {
    const token = this.previous();
    this.consume(TokenType.Semicolon, "Expect ';' after break statement.");
    return new Stmt.Break(token);
  }

  private forStatement(): Stmt {
    this.consume(TokenType.LeftParen, "Expect '(' after 'for'.");
    let initializer: Stmt | null;
    if (this.match(TokenType.Semicolon)) {
      initializer = null;
    } else if (this.match(TokenType.Var)) {
      initializer = this.varDeclaration();
    } else {
      initializer = this.expressionStatement();
    }

    let condition: Expr | null = null;
    if (!this.check(TokenType.Semicolon)) {
      condition = this.expression();
    }
    this.consume(TokenType.Semicolon, "Expect ';' after loop condition.");

    let increment: Expr | null = null;
    if (!this.check(TokenType.RightParen)) {
      increment = this.expression();
    }
    this.consume(TokenType.RightParen, "Expect ')' after for clauses.");
    let body = this.statement();

    if (increment !== null) {
      body = new Stmt.Block([body, new Stmt.Expression(increment)]);
    }

    if (condition === null) condition = new Expr.Literal(true);
    body = new Stmt.While(condition, body);

    if (initializer !== null) {
      body = new Stmt.Block([initializer, body]);
    }

    return body;
  }

  private ifStatement(): Stmt {
    this.consume(TokenType.LeftParen, "Expect '(' after 'if'.");
    const condition = this.expression();
    this.consume(TokenType.RightParen, "Expect '(' after condition.");

    const thenBranch = this.statement();
    const elseBranch = this.match(TokenType.Else) ? this.statement() : null;
    return new Stmt.If(condition, thenBranch, elseBranch);
  }

  private printStatement(): Stmt {
    const value = this.expression();
    this.consume(TokenType.Semicolon, "Expect ';' after value.");
    return new Stmt.Print(value);
  }

  private returnStatement(): Stmt {
    const keyword = this.previous();
    let value = null;
    if (!this.check(TokenType.Semicolon)) {
      value = this.expression();
    }

    this.consume(TokenType.Semicolon, "Expect ';' after return value");
    return new Stmt.Return(keyword, value);
  }

  private whileStatement(): Stmt {
    this.consume(TokenType.LeftParen, "Expect '(' after 'while'.");
    const condition = this.expression();
    this.consume(TokenType.RightParen, "Expect '(' after condition.");
    const body = this.statement();

    return new Stmt.While(condition, body);
  }

  private expressionStatement(): Stmt {
    const expr = this.expression();
    this.consume(TokenType.Semicolon, "Expect ';' after expression.");
    return new Stmt.Expression(expr);
  }

  private function(kind: "function" | "method"): Stmt.Function;
  private function(kind: "expression"): Expr.Function;
  private function(
    kind: "function" | "expression" | "method",
  ): Stmt.Function | Expr.Function {
    let name: Token | null = null;
    if (kind === "function" || kind === "method") {
      name = this.consume(TokenType.Identifier, `Expect ${kind} name.`);
    } else if (this.match(TokenType.Identifier)) {
      name = this.previous();
    }
    let parameters: Token[] | null = null;

    if (kind !== "method" || this.check(TokenType.LeftParen)) {
      this.consume(TokenType.LeftParen, `Expect '(' after ${kind} name.`);
      parameters = [];
      if (!this.check(TokenType.RightParen)) {
        do {
          if (parameters.length >= 255) {
            this.error(this.peek(), "Can't have more than 255 parameters.");
          }

          parameters.push(
            this.consume(TokenType.Identifier, "Expect parameter name."),
          );
        } while (this.match(TokenType.Comma));
      }
      this.consume(TokenType.RightParen, "Expect ')' after parameters.");
    }

    this.consume(TokenType.LeftBrace, `Expect '{' before ${kind} body.`);
    const body = this.block();

    if (kind === "function") {
      return new Stmt.Function(name!, parameters, body);
    }
    return new Expr.Function(name, parameters!, body);
  }

  private block(): Stmt[] {
    const statements: Stmt[] = [];

    while (!this.check(TokenType.RightBrace) && !this.isAtEnd()) {
      statements.push(this.declaration()!);
    }

    this.consume(TokenType.RightBrace, "Expect '}' after block.");
    return statements;
  }

  private assignment(): Expr {
    const expr = this.ternary();

    if (this.match(TokenType.Equal)) {
      const equals = this.previous();
      const value = this.assignment();

      if (expr instanceof Expr.Variable) {
        const { name } = expr;
        return new Expr.Assign(name, value);
      } else if (expr instanceof Expr.Get) {
        return new Expr.Set(expr.object, expr.name, value);
      }

      this.loxError(equals, "Invalid assignment target");
    }

    return expr;
  }

  private ternary(): Expr {
    const expr = this.or();

    if (this.match(TokenType.QuestionMark)) {
      const exprIfTrue = this.ternary();
      this.consume(TokenType.Colon, "Expect ':' after expression");
      const exprIfFalse = this.ternary();
      return new Expr.Ternary(expr, exprIfTrue, exprIfFalse);
    }

    return expr;
  }

  private or(): Expr {
    let expr = this.and();

    while (this.match(TokenType.Or)) {
      const operator = this.previous();
      const right = this.and();
      expr = new Expr.Logical(expr, operator, right);
    }

    return expr;
  }

  private and(): Expr {
    let expr = this.equality();

    while (this.match(TokenType.And)) {
      const operator = this.previous();
      const right = this.equality();
      expr = new Expr.Logical(expr, operator, right);
    }

    return expr;
  }

  private equality(): Expr {
    let expr = this.comparison();

    while (this.match(TokenType.BangEqual, TokenType.EqualEqual)) {
      const operator = this.previous();
      const right = this.comparison();
      expr = new Expr.Binary(expr, operator, right);
    }

    return expr;
  }

  private comparison(): Expr {
    let expr = this.term();

    while (
      this.match(
        TokenType.Greater,
        TokenType.GreaterEqual,
        TokenType.Less,
        TokenType.LessEqual,
      )
    ) {
      const operator = this.previous();
      const right = this.term();
      expr = new Expr.Binary(expr, operator, right);
    }

    return expr;
  }

  private term(): Expr {
    let expr = this.factor();

    while (this.match(TokenType.Minus, TokenType.Plus)) {
      const operator = this.previous();
      const right = this.factor();
      expr = new Expr.Binary(expr, operator, right);
    }

    return expr;
  }

  private factor(): Expr {
    let expr = this.unary();

    while (this.match(TokenType.Slash, TokenType.Star)) {
      const operator = this.previous();
      const right = this.unary();
      expr = new Expr.Binary(expr, operator, right);
    }

    return expr;
  }

  private unary(): Expr {
    if (this.match(TokenType.Bang, TokenType.Minus)) {
      const operator = this.previous();
      const right = this.unary();
      return new Expr.Unary(operator, right);
    }

    return this.call();
  }

  private finishCall(callee: Expr): Expr {
    const args: Expr[] = [];

    if (!this.check(TokenType.RightParen)) {
      do {
        if (args.length >= 255) {
          this.error(this.peek(), "Can't have more than 255 arguments.");
        }
        args.push(this.expression());
      } while (this.match(TokenType.Comma));
    }

    const paren = this.consume(
      TokenType.RightParen,
      "Expect ')' after args.",
    );

    return new Expr.Call(callee, paren, args);
  }

  private call(): Expr {
    let expr = this.primary();

    while (true) {
      if (this.match(TokenType.LeftParen)) {
        expr = this.finishCall(expr);
      } else if (this.match(TokenType.Dot)) {
        const name = this.consume(
          TokenType.Identifier,
          "Expect property name after '.'.",
        );
        expr = new Expr.Get(expr, name);
      } else {
        break;
      }
    }

    return expr;
  }

  private primary(): Expr {
    if (this.match(TokenType.False)) return new Expr.Literal(false);
    if (this.match(TokenType.True)) return new Expr.Literal(true);
    if (this.match(TokenType.Nil)) return new Expr.Literal(null);

    if (this.match(TokenType.Number, TokenType.String)) {
      return new Expr.Literal(this.previous().literal);
    }

    if (this.match(TokenType.This)) return new Expr.This(this.previous());

    if (this.match(TokenType.Identifier)) {
      return new Expr.Variable(this.previous());
    }

    if (this.match(TokenType.LeftParen)) {
      const exprs = [this.expression()];
      while (this.match(TokenType.Comma)) exprs.push(this.expression());
      this.consume(TokenType.RightParen, "Expect ')' after expression.");
      if (exprs.length === 1) return new Expr.Grouping(exprs[0]);
      return new Expr.Comma(exprs);
    }

    if (this.match(TokenType.Fun)) {
      return this.function("expression");
    }

    if (this.handleMalformedBinaryExpression()) {
      return this.expression();
    }

    throw this.error(this.peek(), "Expect expression.");
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

  private error(token: Token, message: string): ParseError {
    this.loxError(token, message);
    return new ParseError();
  }

  private synchronize(): void {
    this.advance();

    while (!this.isAtEnd()) {
      if (this.previous().type === TokenType.Semicolon) return;

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
    if (this.match.call(this, ...this.discardFunctions.keys())) {
      const operator = this.previous();
      this.discardFunctions.get(operator.type)!.call(this);
      this.error(operator, `Expect let hand operand for ${operator.lexeme}`);
      return true;
    }

    return false;
  }
}
