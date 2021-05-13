import Interpreter from "./Interpreter.ts";
import LoxCallable from "./LoxCallable.ts";
import LoxInstance from "./LoxInstance.ts";

export default class LoxClass extends LoxCallable {
  constructor(
    readonly name: string,
  ) {
    super();
  }

  arity(): number {
    return 0;
  }

  call(_interpreter: Interpreter, _args: any[]): any {
    const instance = new LoxInstance(this);
    return instance;
  }

  toString(): string {
    return this.name;
  }
}
