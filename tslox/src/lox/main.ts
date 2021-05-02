import * as fs from 'fs';
import { createInterface } from 'readline';
import AstPrinter from './AstPrinter.js';
import Parser from './Parser.js';
import Scanner from './Scanner.js';
import type Token from './Token.js';
import TokenType from './TokenType.js';

let hadError = false;

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

const run = (text: string): void => {
  const tokens = [...new Scanner(text, error)];
  const parser = new Parser(tokens, error);
  const expression = parser.parse();

  if (hadError || !expression) return;

  console.log(new AstPrinter().print(expression));
};

const runFile = async (fileName: string): Promise<void> => {
  const file = await fs.promises.readFile(fileName, 'utf8');
  run(file);
  if (hadError) process.exit(65);
};

const runPrompt = async (): Promise<void> => {
  const rl = createInterface(process.stdin, process.stdout);
  const questionAsync = (text: string): Promise<string> => new Promise((resolve) => {
    rl.question(text, resolve);
  });
  try {
    while (true) {
      const line = await questionAsync('> ');
      run(line);
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
