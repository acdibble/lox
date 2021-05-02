/* eslint-disable @typescript-eslint/no-namespace */
/* eslint-disable import/export */
import { Expr } from './Expr.js';

export abstract class Stmt {
  abstract accept<T>(visitor: Stmt.Visitor<T>): T;
}

export namespace Stmt {
  export interface Visitor<T> {
    visitExpressionStmt(stmt: Expression): T;
    visitPrintStmt(stmt: Print): T;
  }
}

export class Expression extends Stmt {
  constructor(
    readonly expression: Expr,
  ) {
    super();
  }

  accept<T>(visitor: Stmt.Visitor<T>): T {
    return visitor.visitExpressionStmt(this);
  }
}

export class Print extends Stmt {
  constructor(
    readonly expression: Expr,
  ) {
    super();
  }

  accept<T>(visitor: Stmt.Visitor<T>): T {
    return visitor.visitPrintStmt(this);
  }
}
