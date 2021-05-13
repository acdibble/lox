// deno-lint-ignore-file no-namespace
import type Expr from "./Expr.ts";
import type Token from "./Token.ts";

abstract class Stmt {
  abstract accept<T>(visitor: Stmt.Visitor<T>): T;
}

namespace Stmt {
  export interface Visitor<T> {
    visitBlockStmt(stmt: Stmt.Block): T;
    visitClassStmt(stmt: Stmt.Class): T;
    visitBreakStmt(stmt: Stmt.Break): T;
    visitExpressionStmt(stmt: Stmt.Expression): T;
    visitFunctionStmt(stmt: Stmt.Function): T;
    visitIfStmt(stmt: Stmt.If): T;
    visitPrintStmt(stmt: Stmt.Print): T;
    visitReturnStmt(stmt: Stmt.Return): T;
    visitVarStmt(stmt: Stmt.Var): T;
    visitWhileStmt(stmt: Stmt.While): T;
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

  export class Class extends Stmt {
    constructor(
      readonly name: Token,
      readonly methods: Stmt.Function[],
      readonly classMethods: Stmt.Function[],
    ) {
      super();
    }

    accept<T>(visitor: Stmt.Visitor<T>): T {
      return visitor.visitClassStmt(this);
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
