// deno-lint-ignore-file no-namespace
import type Expr from "./Expr.ts";
import type Token from "./Token.ts";

abstract class Stmt {
  abstract accept<T>(visitor: Stmt.Visitor<T>): T;
}

namespace Stmt {
  export interface Visitor<T> {
    visitBlockStmt(stmt: Block): T;
    visitBreakStmt(stmt: Break): T;
    visitExpressionStmt(stmt: Expression): T;
    visitFunctionStmt(stmt: Function): T;
    visitIfStmt(stmt: If): T;
    visitPrintStmt(stmt: Print): T;
    visitReturnStmt(stmt: Return): T;
    visitVarStmt(stmt: Var): T;
    visitWhileStmt(stmt: While): T;
  }

  export class Block extends Stmt {
    constructor(
      readonly statements: Stmt[],
    ) {
      super();
    }

    accept<T>(visitor: Stmt.Visitor<T>): T {
      return visitor.visitBlockStmt(this);
    }
  }

  export class Break extends Stmt {
    constructor(
      readonly keyword: Token,
    ) {
      super();
    }

    accept<T>(visitor: Stmt.Visitor<T>): T {
      return visitor.visitBreakStmt(this);
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

  export class Function extends Stmt {
    constructor(
      readonly name: Token,
      readonly params: Token[],
      readonly body: Stmt[],
    ) {
      super();
    }

    accept<T>(visitor: Stmt.Visitor<T>): T {
      return visitor.visitFunctionStmt(this);
    }
  }

  export class If extends Stmt {
    constructor(
      readonly condition: Expr,
      readonly thenBranch: Stmt,
      readonly elseBranch: Stmt | null,
    ) {
      super();
    }

    accept<T>(visitor: Stmt.Visitor<T>): T {
      return visitor.visitIfStmt(this);
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

  export class Return extends Stmt {
    constructor(
      readonly keyword: Token,
      readonly value: Expr | null,
    ) {
      super();
    }

    accept<T>(visitor: Stmt.Visitor<T>): T {
      return visitor.visitReturnStmt(this);
    }
  }

  export class Var extends Stmt {
    constructor(
      readonly name: Token,
      readonly initializer: Expr | null,
    ) {
      super();
    }

    accept<T>(visitor: Stmt.Visitor<T>): T {
      return visitor.visitVarStmt(this);
    }
  }

  export class While extends Stmt {
    constructor(
      readonly condition: Expr,
      readonly body: Stmt,
    ) {
      super();
    }

    accept<T>(visitor: Stmt.Visitor<T>): T {
      return visitor.visitWhileStmt(this);
    }
  }
}

export default Stmt;
