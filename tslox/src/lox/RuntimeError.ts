import Token from './Token.js';

export default class RuntimeError extends TypeError {
  constructor(readonly token: Token, message: string) {
    super(message);

    if (!this.stack) {
      Error.captureStackTrace(this, RuntimeError);
    }
  }
}
