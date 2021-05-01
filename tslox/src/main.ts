import * as fs from 'fs';
import { createInterface } from 'readline';
import Scanner from './Scanner.js';

let hadError = false;

const report = (line: number, where: string, message: string): void => {
  console.error(
    `[line ${line}] Error${where}: ${message}`,
  );
  hadError = true;
};

const error = (line: number, message: string): void => {
  report(line, '', message);
};

export type LoxError = typeof error;

const run = (text: string): void => {
  const scanner = new Scanner(text, error);
  for (const token of scanner) {
    console.log(token);
  }
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
