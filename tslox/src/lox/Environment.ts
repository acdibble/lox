/* eslint-disable @typescript-eslint/no-unsafe-return */
/* eslint-disable @typescript-eslint/explicit-module-boundary-types */

import RuntimeError from './RuntimeError.js';
import Token from './Token.js';

/* eslint-disable @typescript-eslint/no-unsafe-assignment */
export default class Environment {
  private readonly values: Record<string, any> = {};

  define(name: string, value: any): void {
    this.values[name] = value;
  }

  get(name: Token): any {
    const value = this.values[name.lexeme];

    if (value !== undefined) return value;

    throw new RuntimeError(name, `Undefined variable '${name.lexeme}'.`);
  }
}
