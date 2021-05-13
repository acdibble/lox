import Interpreter from "./Interpreter.ts";
import LoxCallable from "./LoxCallable.ts";
import LoxFunction from "./LoxFunction.ts";
import LoxInstance from "./LoxInstance.ts";

export default class LoxClass extends LoxCallable {
  constructor(
    readonly name: string,
    private readonly methods: Record<string, LoxFunction>,
  ) {
    super();
  }

  findMethod(name: string): LoxFunction | undefined {
    return this.methods[name];
  }

  arity(): number {
    return this.findMethod("init")?.arity() ?? 0;
  }

  call(interpreter: Interpreter, args: any[]): any {
    const instance = new LoxInstance(this);
    const initializer = this.findMethod("init");
    if (initializer) {
      initializer.bind(instance).call(interpreter, args);
    }
    return instance;
  }

  toString(): string {
    return this.name;
  }
}
