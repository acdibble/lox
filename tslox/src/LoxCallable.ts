import type Interpreter from "./Interpreter.ts";

export default abstract class LoxCallable {
  abstract arity(): number;
  abstract call(interpreter: Interpreter, args: any[]): any;
  abstract toString(): string;
}
