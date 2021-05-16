import type Interpreter from "./Interpreter.ts";
import type LoxCallable from "./LoxCallable.ts";
import type LoxInstance from "./LoxInstance.ts";

interface NativeDeclaration {
  args: string[];
  getter?: boolean;
  initializer?: boolean;
  implementation?: Function;
}

export default class NativeFunction implements LoxCallable {
  constructor(
    private readonly declaration: NativeDeclaration,
    private readonly instance?: LoxInstance,
  ) {
    this.declaration.implementation ??= this.implementation;
  }

  implementation(...args: any[]): any {
    throw new Error("not implemented");
  }

  arity(): number {
    return this.declaration.args.length;
  }

  bind(instance: LoxInstance): NativeFunction {
    return new NativeFunction(this.declaration, instance);
  }

  call(_interpreter: Interpreter, args: any[]) {
    return this.declaration.implementation!.apply(this.instance, args);
  }

  toString(): string {
    return "<native fn>";
  }

  isGetter(): boolean {
    return this.declaration.getter ?? false;
  }

  isInitializer(): boolean {
    return this.declaration.initializer ?? false;
  }
}
