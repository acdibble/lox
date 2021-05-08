import Environment from "./Environment.ts";
import Interpreter from "./Interpreter.ts";
import LoxCallable from "./LoxCallable.ts";
import Stmt from "./Stmt.ts";

export default class LoxFunction extends LoxCallable {
  constructor(
    private readonly declaration: Stmt.Function,
  ) {
    super();
  }

  call(interpreter: Interpreter, args: any[]) {
    const environment = new Environment(interpreter.globals);
    for (let i = 0; i < args.length; i++) {
      environment.define(this.declaration.params[i].lexeme, args[i]);
    }
    interpreter.executeBlock(this.declaration.body, environment);
  }

  arity(): number {
    return this.declaration.params.length;
  }

  toString(): string {
    return `<fn ${this.declaration.name.lexeme}>`;
  }
}
