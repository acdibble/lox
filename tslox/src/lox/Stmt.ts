import Expr from './Expr.js';

abstract class Stmt {
  abstract accept<T>(visitor: Stmt.Visitor<T>): T;
}

namespace Stmt {
  export interface Visitor<T> {
    visitExpressionStmt(stmt: Expression): T;
    visitPrintStmt(stmt: Print): T;
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
}

export default Stmt;
