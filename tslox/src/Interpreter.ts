import Environment from "./Environment.ts";
import Expr from "./Expr.ts";
import LoxCallable from "./LoxCallable.ts";
import LoxFunction from "./LoxFunction.ts";
import { LoxRuntimeError } from "./main.ts";
import RuntimeError from "./RuntimeError.ts";
import Stmt from "./Stmt.ts";
import Return from "./Return.ts";
import Token from "./Token.ts";
import TokenType from "./TokenType.ts";

class BreakError extends Error {}

export default class Interpreter
  implements Expr.Visitor<any>, Stmt.Visitor<void> {
  readonly globals = new Environment();
  private environment = this.globals;

  constructor(
    private readonly loxRuntimeError: LoxRuntimeError,
  ) {
    this.globals.define(
      "clock",
      new class extends LoxCallable {
        arity(): number {
          return 0;
        }

        call(_interpreter: Interpreter, _args: any[]): number {
          return Date.now() / 1000;
        }

        toString(): string {
          return "<native fn>";
        }
      }(),
    );
  }

  interpret(statements: Stmt[]): void {
    try {
      for (const statement of statements) {
        this.execute(statement);
      }
    } catch (error) {
      if (error instanceof RuntimeError) {
        this.loxRuntimeError(error);
      }
      throw error;
    }
  }

  visitAssignExpr(expr: Expr.Assign): any {
    const value = this.evaluate(expr.value);
    this.environment.assign(expr.name, value);
    return value;
  }

  visitBinaryExpr(expr: Expr.Binary): any {
    const left = this.evaluate(expr.left);
    const right = this.evaluate(expr.right);

    switch (expr.operator.type) {
      case TokenType.BangEqual:
        return !this.isEqual(left, right);
      case TokenType.EqualEqual:
        return this.isEqual(left, right);
      case TokenType.Greater:
        this.checkNumberOperands(expr.operator, left, right);
        return left > right;
      case TokenType.GreaterEqual:
        this.checkNumberOperands(expr.operator, left, right);
        return left >= right;
      case TokenType.Less:
        this.checkNumberOperands(expr.operator, left, right);
        return left < right;
      case TokenType.LessEqual:
        this.checkNumberOperands(expr.operator, left, right);
        return left <= right;
      case TokenType.Minus:
        this.checkNumberOperands(expr.operator, left, right);
        return left - right;
      case TokenType.Slash:
        this.checkNumberOperands(expr.operator, left, right);
        if (right === 0) {
          throw new RuntimeError(expr.operator, "Cannot divide by zero.");
        }
        return left / right;
      case TokenType.Star:
        this.checkNumberOperands(expr.operator, left, right);
        return left * right;
      case TokenType.Plus:
        if (typeof left === "string") return left + this.stringify(right);
        if (typeof right === "string") return this.stringify(left) + right;
        this.checkNumberOperands(expr.operator, left, right);
        return (left as number) + (right as number);
      default:
        throw new Error("unreachable");
    }
  }

  visitCallExpr(expr: Expr.Call): any {
    const fn: LoxCallable | unknown = this.evaluate(expr.callee);

    if (!(fn instanceof LoxCallable)) {
      throw new RuntimeError(
        expr.paren,
        "Can only call functions and classes.",
      );
    }

    const args = expr.args.map((arg) => this.evaluate(arg));

    if (args.length != fn.arity()) {
      throw new RuntimeError(
        expr.paren,
        `Expected ${fn.arity()} args but got ${args.length}.`,
      );
    }

    return fn.call(this, args);
  }

  visitCommaExpr(expr: Expr.Comma): any {
    return expr.exprs.reduce((_acc, subexpr) => this.evaluate(subexpr), null);
  }

  visitGroupingExpr(expr: Expr.Grouping): any {
    return this.evaluate(expr.expression);
  }

  visitLiteralExpr(expr: Expr.Literal): any {
    return expr.value;
  }

  visitLogicalExpr(expr: Expr.Logical): any {
    const left = this.evaluate(expr.left);

    if (expr.operator.type === TokenType.Or) {
      if (this.isTruthy(left)) return left;
    } else {
      if (!this.isTruthy(left)) return left;
    }

    return this.evaluate(expr.right);
  }

  visitTernaryExpr(expr: Expr.Ternary): any {
    const condition = this.evaluate(expr.condition);

    if (this.isTruthy(condition)) {
      return this.evaluate(expr.exprIfTrue);
    }

    return this.evaluate(expr.exprIfFalse);
  }

  visitUnaryExpr(expr: Expr.Unary): any {
    const right = this.evaluate(expr.right);

    switch (expr.operator.type) {
      case TokenType.Minus:
        this.checkNumberOperand(expr.operator, right);
        return -(right as number);
      case TokenType.Bang:
        return this.isTruthy(right);
      default:
        throw new Error("unreachable");
    }
  }

  visitVariableExpr(expr: Expr.Variable): any {
    return this.environment.get(expr.name);
  }

  private evaluate(expr: Expr): any {
    return expr.accept(this);
  }

  private execute(stmt: Stmt): void {
    stmt.accept(this);
  }

  executeBlock(statements: Stmt[], environment: Environment): void {
    const previous = this.environment;
    try {
      this.environment = environment;

      for (const statement of statements) {
        this.execute(statement);
      }
    } finally {
      this.environment = previous;
    }
  }

  visitBlockStmt(stmt: Stmt.Block): void {
    this.executeBlock(stmt.statements, new Environment(this.environment));
  }

  visitBreakStmt(_stmt: Stmt.Break): void {
    throw new BreakError();
  }

  visitExpressionStmt(stmt: Stmt.Expression): void {
    this.evaluate(stmt.expression);
  }

  visitFunctionStmt(stmt: Stmt.Function): void {
    const fn = new LoxFunction(stmt);
    this.environment.define(stmt.name.lexeme, fn);
  }

  visitIfStmt(stmt: Stmt.If): void {
    if (this.isTruthy(this.evaluate(stmt.condition))) {
      this.execute(stmt.thenBranch);
    } else if (stmt.elseBranch) {
      this.execute(stmt.elseBranch);
    }
  }

  visitPrintStmt(stmt: Stmt.Print): void {
    const result = this.evaluate(stmt.expression);
    console.log(this.stringify(result));
  }

  visitReturnStmt(stmt: Stmt.Return): void {
    const value = stmt.value && this.evaluate(stmt.value);
    throw new Return(value);
  }

  visitVarStmt(stmt: Stmt.Var): void {
    let value = Symbol.for("uninitialized");
    if (stmt.initializer !== null) value = this.evaluate(stmt.initializer);
    this.environment.define(stmt.name.lexeme, value);
  }

  visitWhileStmt(stmt: Stmt.While): void {
    try {
      while (this.isTruthy(this.evaluate(stmt.condition))) {
        this.execute(stmt.body);
      }
    } catch (error) {
      if (!(error instanceof BreakError)) {
        throw error;
      }
    }
  }

  private isTruthy(value: any): boolean {
    if (value === null) return false;
    if (typeof value === "boolean") return value;
    return true;
  }

  private isEqual(a: any, b: any): boolean {
    return a === b;
  }

  private stringify(value: any): string {
    if (value == null) return "nil";

    return value.toString!();
  }

  private checkNumberOperand(operator: Token, operand: any): void {
    if (typeof operand === "number") return;
    throw new RuntimeError(operator, "Operand must be a number");
  }

  private checkNumberOperands(operator: Token, left: any, right: any): void {
    if (typeof left === "number" && typeof right === "number") return;
    throw new RuntimeError(operator, "Operands must be numbers");
  }
}
