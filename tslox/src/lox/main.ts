import * as fs from 'fs';
import { createInterface } from 'readline';
import Expr from './Expr.js';
import Interpreter from './Interpreter.js';
import Parser from './Parser.js';
import type RuntimeError from './RuntimeError.js';
import Scanner from './Scanner.js';
import Stmt from './Stmt.js';
import Token from './Token.js';
import TokenType from './TokenType.js';

let hadError = false;
let hadRuntimeError = false;

const report = (line: number, where: string, message: string): void => {
  console.error(`[line ${line}] Error${where}: ${message}`);
  hadError = true;
};

function error(token: Token, message: string): void;
function error(line: number, message: string): void;
function error(arg0: Token | number, message: string): void {
  if (typeof arg0 === 'number') {
    report(arg0, '', message);
    return;
  }

  if (arg0.type === TokenType.EOF) {
    report(arg0.line, ' at end', message);
  } else {
    report(arg0.line, ` at '${arg0.lexeme}'`, message);
  }
}

export type LoxError = typeof error;

const runtimeError = (err: RuntimeError): void => {
  console.error(`${err.message}\n[line ${err.token.line}]`);
  hadRuntimeError = true;
};

const interpreter = new Interpreter(runtimeError);

export type LoxRuntimeError = typeof runtimeError;

enum Mode {
  File,
  REPL,
}

const run = (text: string, mode = Mode.File): void => {
  const tokens = [...new Scanner(text, error)];
  const parser = new Parser(tokens, error);
  const statements = parser.parse();
  if (hadError || !statements.length) return;
  let finalStmt: Stmt.Expression | null = null;
  if (mode === Mode.REPL && statements[statements.length - 1] instanceof Stmt.Expression) {
    finalStmt = statements.pop() as Stmt.Expression;
  }
  interpreter.interpret(statements);
  if (mode === Mode.REPL) {
    const token = new Token(TokenType.Identifier, '_', null, 1);
    interpreter.interpret([
      new Stmt.Var(token, finalStmt && finalStmt.expression),
      new Stmt.Print(new Expr.Variable(token)),
    ]);
  }
};

const runFile = async (fileName: string): Promise<void> => {
  const file = await fs.promises.readFile(fileName, 'utf8');
  run(file);
  if (hadError) process.exit(65);
  if (hadRuntimeError) process.exit(70);
};

const runPrompt = async (): Promise<void> => {
  const rl = createInterface(process.stdin, process.stdout);
  const questionAsync = (text: string): Promise<string> => new Promise((resolve) => {
    rl.question(text, resolve);
  });
  try {
    while (true) {
      let line = await questionAsync('> ');
      if (!line.endsWith(';')) line += ';';
      try {
        run(line, Mode.REPL);
      } catch {
        //
      }
      hadError = false;
    }
  } finally {
    rl.close();
  }
};

const main = async (args = process.argv.slice(2)): Promise<void> => {
  if (args.length > 1) {
    console.error('Usage: tslox [script]');
    process.exit(64);
  } else if (args.length === 1) {
    await runFile(args[0]!);
  } else {
    await runPrompt();
  }
};

// eslint-disable-next-line @typescript-eslint/no-floating-promises
main();
