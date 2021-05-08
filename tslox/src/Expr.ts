// deno-lint-ignore-file no-namespace
import Token from "./Token.ts";

abstract class Expr {
  abstract accept<T>(visitor: Expr.Visitor<T>): T;
}

namespace Expr {
  export interface Visitor<T> {
    visitAssignExpr(expr: Assign): T;
    visitBinaryExpr(expr: Binary): T;
    visitCommaExpr(expr: Comma): T;
    visitGroupingExpr(expr: Grouping): T;
    visitLiteralExpr(expr: Literal): T;
    visitLogicalExpr(expr: Logical): T;
    visitTernaryExpr(expr: Ternary): T;
    visitUnaryExpr(expr: Unary): T;
    visitVariableExpr(expr: Variable): T;
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

  export class Comma extends Expr {
    constructor(
      readonly exprs: Expr[],
    ) {
      super();
    }

    accept<T>(visitor: Expr.Visitor<T>): T {
      return visitor.visitCommaExpr(this);
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
