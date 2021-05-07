import Expr from './Expr.js';
import Stmt from './Stmt.js';

export default class AstPrinter implements Expr.Visitor<string>, Stmt.Visitor<string> {
  print(stmts: Stmt[]): string {
    return stmts.map((stmt) => stmt.accept(this)).join('\n');
  }

  private parenthesize(name: string, ...exprs: Expr[]): string {
    return `(${name} ${exprs.map((expr) => expr.accept(this)).join(' ')})`;
  }

  visitBinaryExpr(expr: Expr.Binary): string {
    return this.parenthesize(expr.operator.lexeme, expr.left, expr.right);
  }

  visitGroupingExpr(expr: Expr.Grouping): string {
    return this.parenthesize('group', expr.expression);
  }

  // eslint-disable-next-line class-methods-use-this
  visitLiteralExpr(expr: Expr.Literal): string {
    if (expr.value === null) return 'nil';
    return expr.value.toString();
  }

  visitUnaryExpr(expr: Expr.Unary): string {
    return this.parenthesize(expr.operator.lexeme, expr.right);
  }

  visitCommaExpr(expr: Expr.Comma): string {
    return this.parenthesize('comma', ...expr.exprs);
  }

  visitTernaryExpr(expr: Expr.Ternary): string {
    return this.parenthesize('ternary', expr.condition, expr.exprIfTrue, expr.exprIfFalse);
  }

  visitVariableExpr(expr: Expr.Variable): string {
    return this.parenthesize(`variable ${expr.name}`);
  }

  visitAssignExpr(expr: Expr.Assign): string {
    return this.parenthesize('assign', expr, expr.value);
  }

  visitBlockStmt(stmt: Stmt.Block): string {
    return this.parenthesize(`block ${this.print(stmt.statements)}`);
  }

  visitExpressionStmt(stmt: Stmt.Expression): string {
    return this.parenthesize('expression', stmt.expression);
  }

  visitPrintStmt(stmt: Stmt.Print): string {
    return this.parenthesize('print', stmt.expression);
  }

  visitVarStmt(stmt: Stmt.Var): string {
    if (stmt.initializer === null) return this.parenthesize(`var ${stmt.name.lexeme} = nil`);
    return this.parenthesize(`var ${stmt.name.lexeme} =`, stmt.initializer);
  }
}
