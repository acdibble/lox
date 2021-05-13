import Expr from "./Expr.ts";
import type Interpreter from "./Interpreter.ts";
import type { LoxError } from "./main.ts";
import type Stmt from "./Stmt.ts";
import type Token from "./Token.ts";

class Stack<T> {
  private readonly storage: T[] = [];

  get(index: number): T | undefined {
    return this.storage[index];
  }

  isEmpty(): boolean {
    return this.storage.length === 0;
  }

  push(value: T): void {
    this.storage.push(value);
  }

  pop(): T | undefined {
    return this.storage.pop();
  }

  peek(cb: (value: T) => void): void {
    const value = this.storage[this.storage.length - 1];
    if (value) cb(value);
  }

  get size() {
    return this.storage.length;
  }
}

const enum FunctionType {
  None,
  Function,
}

const enum LoopType {
  None,
  While,
}

const enum VariableState {
  Declared,
  Defined,
  Read,
}

export default class Resolver
  implements Expr.Visitor<void>, Stmt.Visitor<void> {
  private readonly scopes: Stack<
    Map<string, { token: Token; state: VariableState }>
  > = new Stack();
  private currentFunction = FunctionType.None;
  private currentLoop = LoopType.None;

  constructor(
    private readonly interpreter: Interpreter,
    private readonly loxError: LoxError,
  ) {}

  private _resolve(value: Stmt | Expr): void {
    value.accept(this);
  }

  resolve(statements: Stmt[]): void {
    for (const statement of statements) {
      this._resolve(statement);
    }
  }

  private resolveFunction(
    fn: Stmt.Function | Expr.Function,
    type: FunctionType,
  ): void {
    const enclosingFunction = this.currentFunction;
    this.currentFunction = type;
    this.beginScope();
    for (const param of fn.params) {
      this.declare(param);
      this.define(param);
    }
    this.resolve(fn.body);
    this.endScope();
    this.currentFunction = enclosingFunction;
  }

  private beginScope(): void {
    this.scopes.push(new Map());
  }

  private endScope(): void {
    for (const { token, state } of this.scopes.pop()!.values()) {
      if (state === VariableState.Defined) {
        this.loxError(token, "Unused local variable.");
      }
    }
  }

  /** @private */
  declare(name: Token): void {
    this.scopes.peek((scope) => {
      if (scope.has(name.lexeme)) {
        this.loxError(name, "Already variable with this name in this scope.");
      }
      scope.set(name.lexeme, { token: name, state: VariableState.Declared });
    });
  }

  private define(name: Token): void {
    this.scopes.peek((scope) => {
      scope.get(name.lexeme)!.state = VariableState.Defined;
    });
  }

  private resolveLocal(expr: Expr, name: Token): void {
    for (let i = this.scopes.size - 1; i >= 0; i--) {
      const variable = this.scopes.get(i)!.get(name.lexeme);
      if (variable) {
        this.interpreter.resolve(expr, this.scopes.size - 1 - i);
        if (!(expr instanceof Expr.Assign)) variable.state = VariableState.Read;
        return;
      }
    }
  }

  visitBlockStmt(stmt: Stmt.Block): void {
    this.beginScope();
    this.resolve(stmt.statements);
    this.endScope();
  }

  visitBreakStmt(stmt: Stmt.Break): void {
    if (this.currentLoop === LoopType.None) {
      this.loxError(stmt.keyword, "Must be inside a loop to use 'break'.");
    }
  }

  visitClassStmt(stmt: Stmt.Class): void {
    this.declare(stmt.name);
    this.define(stmt.name);
  }

  visitExpressionStmt(stmt: Stmt.Expression): void {
    this._resolve(stmt.expression);
  }

  visitFunctionStmt(stmt: Stmt.Function): void {
    this.declare(stmt.name);
    this.define(stmt.name);
    this.resolveFunction(stmt, FunctionType.Function);
  }

  visitIfStmt(stmt: Stmt.If): void {
    this._resolve(stmt.condition);
    this._resolve(stmt.thenBranch);
    if (stmt.elseBranch) this._resolve(stmt.elseBranch);
  }

  visitPrintStmt(stmt: Stmt.Print): void {
    this._resolve(stmt.expression);
  }

  visitReturnStmt(stmt: Stmt.Return): void {
    if (this.currentFunction === FunctionType.None) {
      this.loxError(stmt.keyword, "Can't return from top-level code.");
    }
    if (stmt.value) this._resolve(stmt.value);
  }

  visitVarStmt(stmt: Stmt.Var): void {
    this.declare(stmt.name);
    if (stmt.initializer) {
      this._resolve(stmt.initializer);
    }
    this.define(stmt.name);
  }

  visitWhileStmt(stmt: Stmt.While): void {
    const enclosingLoop = this.currentLoop;
    this.currentLoop = LoopType.While;
    this._resolve(stmt.condition);
    this._resolve(stmt.body);
    this.currentLoop = enclosingLoop;
  }

  visitAssignExpr(expr: Expr.Assign): void {
    this._resolve(expr.value);
    this.resolveLocal(expr, expr.name);
  }

  visitBinaryExpr(expr: Expr.Binary): void {
    this._resolve(expr.left);
    this._resolve(expr.right);
  }

  visitCallExpr(expr: Expr.Call): void {
    this._resolve(expr.callee);

    for (const arg of expr.args) {
      this._resolve(arg);
    }
  }

  visitCommaExpr(expr: Expr.Comma): void {
    for (const ex of expr.expressions) {
      this._resolve(ex);
    }
  }

  visitFunctionExpr(expr: Expr.Function): void {
    this.resolveFunction(expr, FunctionType.Function);
  }

  visitGroupingExpr(expr: Expr.Grouping): void {
    this._resolve(expr.expression);
  }

  visitLiteralExpr(_expr: Expr.Literal): void {
  }

  visitLogicalExpr(expr: Expr.Logical): void {
    this._resolve(expr.left);
    this._resolve(expr.right);
  }

  visitTernaryExpr(expr: Expr.Ternary): void {
    this._resolve(expr.condition);
    this._resolve(expr.exprIfTrue);
    this._resolve(expr.exprIfFalse);
  }

  visitUnaryExpr(expr: Expr.Unary): void {
    this._resolve(expr.right);
  }

  visitVariableExpr(expr: Expr.Variable): void {
    this.scopes.peek((scope) => {
      if (scope.get(expr.name.lexeme)?.state === VariableState.Declared) {
        this.loxError(
          expr.name,
          "Can't read local variable in its own initializer.",
        );
      }

      this.resolveLocal(expr, expr.name);
    });
  }
}
