import Environment from "./Environment.ts";
import Expr from "./Expr.ts";
import Interpreter from "./Interpreter.ts";
import LoxCallable from "./LoxCallable.ts";
import type LoxInstance from "./LoxInstance.ts";
import Stmt from "./Stmt.ts";
import Return from "./Return.ts";

export default class LoxFunction implements LoxCallable {
  constructor(
    private readonly declaration: Stmt.Function | Expr.Function,
    private readonly closure: Environment,
    private readonly isInitializer: boolean,
  ) {}

  bind(instance: LoxInstance): LoxFunction {
    const environment = new Environment(this.closure);
    environment.define("this", instance);
    return new LoxFunction(this.declaration, environment, this.isInitializer);
  }

  call(interpreter: Interpreter, args: any[] | null): any {
    const environment = new Environment(this.closure);
    if (this.declaration.params) {
      for (let i = 0; i < args!.length; i++) {
        environment.define(this.declaration.params[i].lexeme, args![i]);
      }
    }
    try {
      interpreter.executeBlock(this.declaration.body, environment);
    } catch (returnValue) {
      if (returnValue instanceof Return) {
        if (this.isInitializer) return this.closure.getAt(0, "this");
        return returnValue.value;
      }
      throw returnValue;
    }
    if (this.isInitializer) return this.closure.getAt(0, "this");
    return null;
  }

  arity(): number {
    return this.declaration.params?.length ?? 0;
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

  isGetter(): boolean {
    return this.declaration.params === null;
  }
}
