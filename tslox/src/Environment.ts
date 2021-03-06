import RuntimeError from "./RuntimeError.ts";
import Token from "./Token.ts";

export default class Environment {
  private readonly values: Map<string, any> = new Map();

  constructor(
    readonly enclosing?: Environment,
  ) {}

  define(name: string, value: any): void {
    this.values.set(name, value);
  }

  ancestor(distance: number): Environment {
    let environment: Environment = this;

    for (let i = 0; i < distance; i++) {
      environment = environment.enclosing!;
    }

    return environment;
  }

  getAt(distance: number, name: string): any {
    return this.ancestor(distance).values.get(name);
  }

  assignAt(distance: number, name: Token, value: any): void {
    this.ancestor(distance).values.set(name.lexeme, value);
  }

  get(name: Token): any {
    const value = this.values.get(name.lexeme);
    if (value === Symbol.for("uninitialized")) {
      throw new RuntimeError(
        name,
        `Uninitialized variable '${name.lexeme}'.`,
      );
    }

    if (value !== undefined) return value;

    if (this.enclosing) return this.enclosing.get(name);

    throw new RuntimeError(name, `Undefined variable '${name.lexeme}'.`);
  }

  assign(name: Token, value: any): void {
    if (this.values.has(name.lexeme)) {
      this.values.set(name.lexeme, value);
      return;
    }

    if (this.enclosing) {
      this.enclosing.assign(name, value);
      return;
    }

    throw new RuntimeError(name, `Undefined variable '${name.lexeme}'.`);
  }
}
