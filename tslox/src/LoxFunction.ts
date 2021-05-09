import Environment from "./Environment.ts";
import Expr from "./Expr.ts";
import Interpreter from "./Interpreter.ts";
import LoxCallable from "./LoxCallable.ts";
import Stmt from "./Stmt.ts";
import Return from "./Return.ts";

export default class LoxFunction extends LoxCallable {
  constructor(
    private readonly declaration: Stmt.Function | Expr.Function,
    private readonly closure: Environment,
  ) {
    super();
  }

  call(interpreter: Interpreter, args: any[]): any {
    const environment = new Environment(this.closure);
    for (let i = 0; i < args.length; i++) {
      environment.define(this.declaration.params[i].lexeme, args[i]);
    }
    try {
      interpreter.executeBlock(this.declaration.body, environment);
    } catch (returnValue) {
      if (returnValue instanceof Return) return returnValue.value;
      throw returnValue;
    }
    return null;
  }

  arity(): number {
    return this.declaration.params.length;
  }

  toString(): string {
    let name: string;
    if (this.declaration instanceof Expr.Function) {
      name = this.declaration.name?.lexeme ?? "(anonymous)";
    } else {
      name = this.declaration.name.lexeme;
    }
    return `<fn ${name}>`;
  }
}
