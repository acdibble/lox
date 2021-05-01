import {
  Binary,
  Expr,
  Grouping,
  Literal,
  Unary,
  Visitor,
} from './Expr.js';
import Token from './Token.js';
import TokenType from './TokenType.js';

export default class AstPrinter implements Visitor<string> {
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
}

const main = (): void => {
  const expr = new Binary(
    new Unary(
      new Token(TokenType.Minus, '-', null, 1),
      new Literal(123),
    ),
    new Token(TokenType.Star, '*', null, 1),
    new Grouping(new Literal(45.67)),
  );

  console.log(new AstPrinter().print(expr));
};

main();
