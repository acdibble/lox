import Expr from './Expr.js';

export default class AstPrinter implements Expr.Visitor<string> {
  visitVariableExpr(expr: Expr.Variable): string {
    throw new Error('Method not implemented.');
  }

  print(expr: Expr): string {
    return expr.accept(this);
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
}
