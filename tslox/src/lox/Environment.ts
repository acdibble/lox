/* eslint-disable @typescript-eslint/no-unsafe-return */
/* eslint-disable @typescript-eslint/explicit-module-boundary-types */

import RuntimeError from './RuntimeError.js';
import Token from './Token.js';

/* eslint-disable @typescript-eslint/no-unsafe-assignment */
export default class Environment {
  private readonly values: Map<string, any> = new Map();

  define(name: string, value: any): void {
    this.values.set(name, value);
  }

  get(name: Token): any {
    const value = this.values.get(name.lexeme);

    if (value !== undefined) return value;

    throw new RuntimeError(name, `Undefined variable '${name.lexeme}'.`);
  }

  assign(name: Token, value: any): void {
    if (this.values.has(name.lexeme)) {
      this.values.set(name.lexeme, value);
      return;
    }

    throw new RuntimeError(name, `Undefined variable '${name.lexeme}'.`);
  }
}
