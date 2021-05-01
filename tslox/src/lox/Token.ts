/* eslint-disable @typescript-eslint/explicit-module-boundary-types */
import TokenType from './TokenType.js';

export default class Token {
  constructor(type: TokenType.String, lexeme: string, literal: string, line: number)
  constructor(type: TokenType.Number, lexeme: string, literal: number, line: number)
  constructor(type: TokenType, lexeme: string, literal: null, line: number)
  constructor(
    readonly type: TokenType,
    readonly lexeme: string,
    readonly literal: any,
    readonly line: number,
  ) {}

  toString(): string {
    return `${this.type} ${this.lexeme} ${this.literal}`;
  }
}
