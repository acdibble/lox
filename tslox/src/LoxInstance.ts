import type LoxClass from "./LoxClass.ts";
import RuntimeError from "./RuntimeError.ts";
import type Token from "./Token.ts";

export default class LoxInstance {
  private readonly fields: Record<string, any> = Object.create(null);

  constructor(
    protected readonly klass: LoxClass,
  ) {}

  get(name: Token): any {
    if (name.lexeme in this.fields) {
      return this.fields[name.lexeme];
    }

    const method = this.klass.findMethod(name.lexeme);
    if (method) return method.bind(this);

    throw new RuntimeError(name, `Undefined property '${name.lexeme}'.`);
  }

  set(name: Token, value: any): void {
    this.fields[name.lexeme] = value;
  }

  toString(): string {
    return `${this.klass.name} instance`;
  }
}
