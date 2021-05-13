import type LoxClass from "./LoxClass.ts";

export default class LoxInstance {
  constructor(
    private klass: LoxClass,
  ) {}

  toString(): string {
    return `${this.klass.name} instance`;
  }
}
