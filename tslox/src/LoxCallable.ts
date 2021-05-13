import type Interpreter from "./Interpreter.ts";

export default interface LoxCallable {
  arity(): number;
  call(interpreter: Interpreter, args: any[]): any;
  toString(): string;
}

export const implementsLoxCallable = (obj: unknown): obj is LoxCallable => {
  if (typeof obj !== "object" || obj === null) return false;
  // @ts-expect-error ignore
  if (typeof obj.arity !== "function" || obj.arity.length !== 0) return false;
  // @ts-expect-error ignore
  if (typeof obj.call !== "function" || obj.call.length !== 2) return false;
  if (typeof obj.toString !== "function" || obj.toString.length !== 0) {
    return false;
  }
  return true;
};
