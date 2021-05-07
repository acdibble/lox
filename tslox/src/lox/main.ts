import { readLines } from "https://deno.land/std@0.95.0/io/mod.ts";
import Expr from "./Expr.ts";
import Interpreter from "./Interpreter.ts";
import Parser from "./Parser.ts";
import type RuntimeError from "./RuntimeError.ts";
import Scanner from "./Scanner.ts";
import Stmt from "./Stmt.ts";
import Token from "./Token.ts";
import TokenType from "./TokenType.ts";

let hadError = false;
let hadRuntimeError = false;

const report = (line: number, where: string, message: string): void => {
  console.error(`[line ${line}] Error${where}: ${message}`);
  hadError = true;
};

function error(token: Token, message: string): void;
function error(line: number, message: string): void;
function error(arg0: Token | number, message: string): void {
  if (typeof arg0 === "number") {
    report(arg0, "", message);
    return;
  }

  if (arg0.type === TokenType.EOF) {
    report(arg0.line, " at end", message);
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
  if (
    mode === Mode.REPL &&
    statements[statements.length - 1] instanceof Stmt.Expression
  ) {
    finalStmt = statements.pop() as Stmt.Expression;
  }
  interpreter.interpret(statements);
  if (mode === Mode.REPL) {
    const token = new Token(TokenType.Identifier, "_", null, 1);
    interpreter.interpret([
      new Stmt.Var(token, finalStmt && finalStmt.expression),
      new Stmt.Print(new Expr.Variable(token)),
    ]);
  }
};

const runFile = async (fileName: string): Promise<void> => {
  const file = await Deno.readTextFile(fileName);
  run(file);
  if (hadError) Deno.exit(65);
  if (hadRuntimeError) Deno.exit(70);
};

const runPrompt = async (): Promise<void> => {
  console.log("> ");
  for await (let line of readLines(Deno.stdin)) {
    if (!line.endsWith(";")) line += ";";
    try {
      run(line, Mode.REPL);
    } catch {
      //
    }
    hadError = false;
  }
};

const main = async (args = Deno.args): Promise<void> => {
  if (args.length > 1) {
    console.error("Usage: tslox [script]");
    Deno.exit(64);
  } else if (args.length === 1) {
    await runFile(args[0]!);
  } else {
    await runPrompt();
  }
};

// eslint-disable-next-line @typescript-eslint/no-floating-promises
main();
