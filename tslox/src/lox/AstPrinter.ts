import {
  Binary,
  Comma,
  Expr,
  Grouping,
  Literal,
  Ternary,
  Unary,
} from './Expr.js';

export default class AstPrinter implements Expr.Visitor<string> {
  print(expr: Expr): string {
    return expr.accept(this);
  }

  private parenthesize(name: string, ...exprs: Expr[]): string {
    return `(${name} ${exprs.map((expr) => expr.accept(this)).join(' ')})`;
  }

  visitBinaryExpr(expr: Binary): string {
    return this.parenthesize(expr.operator.lexeme, expr.left, expr.right);
  }

  visitGroupingExpr(expr: Grouping): string {
    return this.parenthesize('group', expr.expression);
  }

  // eslint-disable-next-line class-methods-use-this
  visitLiteralExpr(expr: Literal): string {
    if (expr.value === null) return 'nil';
    return expr.value.toString();
  }

  visitUnaryExpr(expr: Unary): string {
    return this.parenthesize(expr.operator.lexeme, expr.right);
  }

  visitCommaExpr(expr: Comma): string {
    return this.parenthesize('comma', ...expr.exprs);
  }

  visitTernaryExpr(expr: Ternary): string {
    return this.parenthesize('ternary', expr.condition, expr.exprIfTrue, expr.exprIfFalse);
  }
}
