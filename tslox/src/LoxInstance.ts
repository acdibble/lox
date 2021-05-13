import type LoxClass from "./LoxClass.ts";
import RuntimeError from "./RuntimeError.ts";
import type Token from "./Token.ts";

export default class LoxInstance {
  private readonly fields: Record<string, any> = {};

  constructor(
    private klass: LoxClass,
  ) {}

  get(name: Token): any {
    const value = this.fields[name.lexeme];
    if (value) return value;

    throw new RuntimeError(name, `Undefined property '${name.lexeme}'.`);
  }

  set(name: Token, value: any): void {
    this.fields[name.lexeme] = value;
  }

  toString(): string {
    return `${this.klass.name} instance`;
  }
}
