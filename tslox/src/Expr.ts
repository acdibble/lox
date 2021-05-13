// deno-lint-ignore-file no-namespace
import type Stmt from "./Stmt.ts";
import type Token from "./Token.ts";

abstract class Expr {
  abstract accept<T>(visitor: Expr.Visitor<T>): T;
}

namespace Expr {
  export interface Visitor<T> {
    visitAssignExpr(expr: Expr.Assign): T;
    visitBinaryExpr(expr: Expr.Binary): T;
    visitCallExpr(expr: Expr.Call): T;
    visitCommaExpr(expr: Expr.Comma): T;
    visitFunctionExpr(expr: Expr.Function): T;
    visitGetExpr(expr: Expr.Get): T;
    visitGroupingExpr(expr: Expr.Grouping): T;
    visitLiteralExpr(expr: Expr.Literal): T;
    visitLogicalExpr(expr: Expr.Logical): T;
    visitSetExpr(expr: Expr.Set): T;
    visitThisExpr(expr: Expr.This): T;
    visitTernaryExpr(expr: Expr.Ternary): T;
    visitUnaryExpr(expr: Expr.Unary): T;
    visitVariableExpr(expr: Expr.Variable): T;
  }

  export class Assign extends Expr {
    constructor(
      readonly name: Token,
      readonly value: Expr,
    ) {
      super();
    }

    accept<T>(visitor: Expr.Visitor<T>): T {
      return visitor.visitAssignExpr(this);
    }
  }

  export class Binary extends Expr {
    constructor(
      readonly left: Expr,
      readonly operator: Token,
      readonly right: Expr,
    ) {
      super();
    }

    accept<T>(visitor: Expr.Visitor<T>): T {
      return visitor.visitBinaryExpr(this);
    }
  }

  export class Call extends Expr {
    constructor(
      readonly callee: Expr,
      readonly paren: Token,
      readonly args: Expr[],
    ) {
      super();
    }

    accept<T>(visitor: Expr.Visitor<T>): T {
      return visitor.visitCallExpr(this);
    }
  }

  export class Comma extends Expr {
    constructor(
      readonly expressions: Expr[],
    ) {
      super();
    }

    accept<T>(visitor: Expr.Visitor<T>): T {
      return visitor.visitCommaExpr(this);
    }
  }

  export class Function extends Expr {
    constructor(
      readonly name: Token | null,
      readonly params: Token[],
      readonly body: Stmt[],
    ) {
      super();
    }

    accept<T>(visitor: Expr.Visitor<T>): T {
      return visitor.visitFunctionExpr(this);
    }
  }

  export class Get extends Expr {
    constructor(
      readonly object: Expr,
      readonly name: Token,
    ) {
      super();
    }

    accept<T>(visitor: Expr.Visitor<T>): T {
      return visitor.visitGetExpr(this);
    }
  }

  export class Grouping extends Expr {
    constructor(
      readonly expression: Expr,
    ) {
      super();
    }

    accept<T>(visitor: Expr.Visitor<T>): T {
      return visitor.visitGroupingExpr(this);
    }
  }

  export class Literal extends Expr {
    constructor(
      readonly value: { toString(): string } | null,
    ) {
      super();
    }

    accept<T>(visitor: Expr.Visitor<T>): T {
      return visitor.visitLiteralExpr(this);
    }
  }

  export class Logical extends Expr {
    constructor(
      readonly left: Expr,
      readonly operator: Token,
      readonly right: Expr,
    ) {
      super();
    }

    accept<T>(visitor: Expr.Visitor<T>): T {
      return visitor.visitLogicalExpr(this);
    }
  }

  export class Set extends Expr {
    constructor(
      readonly object: Expr,
      readonly name: Token,
      readonly value: Expr,
    ) {
      super();
    }

    accept<T>(visitor: Expr.Visitor<T>): T {
      return visitor.visitSetExpr(this);
    }
  }

  export class This extends Expr {
    constructor(
      readonly keyword: Token,
    ) {
      super();
    }

    accept<T>(visitor: Expr.Visitor<T>): T {
      return visitor.visitThisExpr(this);
    }
  }

  export class Ternary extends Expr {
    constructor(
      readonly condition: Expr,
      readonly exprIfTrue: Expr,
      readonly exprIfFalse: Expr,
    ) {
      super();
    }

    accept<T>(visitor: Expr.Visitor<T>): T {
      return visitor.visitTernaryExpr(this);
    }
  }

  export class Unary extends Expr {
    constructor(
      readonly operator: Token,
      readonly right: Expr,
    ) {
      super();
    }

    accept<T>(visitor: Expr.Visitor<T>): T {
      return visitor.visitUnaryExpr(this);
    }
  }

  export class Variable extends Expr {
    constructor(
      readonly name: Token,
    ) {
      super();
    }

    accept<T>(visitor: Expr.Visitor<T>): T {
      return visitor.visitVariableExpr(this);
    }
  }
}

export default Expr;
