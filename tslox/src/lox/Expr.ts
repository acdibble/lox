import Token from './Token.js';

export abstract class Expr {
  abstract accept<T>(visitor: Visitor<T>): T;
}

export interface Visitor<T> {
  visitBinaryExpr(expr: Binary): T;
  visitGroupingExpr(expr: Grouping): T;
  visitLiteralExpr(expr: Literal): T;
  visitUnaryExpr(expr: Unary): T;
  visitCommaExpr(expr: Comma): T;
  visitTernaryExpr(expr: Ternary): T;
}

export class Binary extends Expr {
  constructor(
    readonly left: Expr,
    readonly operator: Token,
    readonly right: Expr,
  ) {
    super();
  }

  accept<T>(visitor: Visitor<T>): T {
    return visitor.visitBinaryExpr(this);
  }
}

export class Grouping extends Expr {
  constructor(
    readonly expression: Expr,
  ) {
    super();
  }

  accept<T>(visitor: Visitor<T>): T {
    return visitor.visitGroupingExpr(this);
  }
}

export class Literal extends Expr {
  constructor(
    readonly value: { toString(): string } | null,
  ) {
    super();
  }

  accept<T>(visitor: Visitor<T>): T {
    return visitor.visitLiteralExpr(this);
  }
}

export class Unary extends Expr {
  constructor(
    readonly operator: Token,
    readonly right: Expr,
  ) {
    super();
  }

  accept<T>(visitor: Visitor<T>): T {
    return visitor.visitUnaryExpr(this);
  }
}

export class Comma extends Expr {
  constructor(
    readonly exprs: Expr[],
  ) {
    super();
  }

  accept<T>(visitor: Visitor<T>): T {
    return visitor.visitCommaExpr(this);
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

  accept<T>(visitor: Visitor<T>): T {
    return visitor.visitTernaryExpr(this);
  }
}
