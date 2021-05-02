import * as fs from 'fs';

const keys = Object.keys as <T>(obj: T) => (keyof T)[];
const entries = Object.entries as <T>(obj: T) => [keyof T, T[keyof T]][];

const defineAst = async (
  handle: fs.promises.FileHandle,
  baseName: string,
  classes: Record<string, Record<string, string>>,
): Promise<void> => {
  await handle.write(`import Token from './Token.js';

export abstract class ${baseName} {
  abstract accept<T>(visitor: Visitor<T>): T;
}\n`);

  await handle.write('\nexport interface Visitor<T> {\n');

  for (const className of keys(classes)) {
    await handle.write(`  visit${className}${baseName}(${baseName.toLowerCase()}: ${className}): T;\n`);
  }

  await handle.write('}\n');

  for (const [className, args] of entries(classes)) {
    await handle.write(`\nexport class ${className} extends Expr {
  constructor(\n`);
    for (const [name, type] of entries(args)) {
      await handle.write(`    readonly ${name}: ${type},\n`);
    }
    await handle.write('  ) {\n');
    await handle.write('    super();\n');
    await handle.write('  }\n');
    await handle.write('\n  accept<T>(visitor: Visitor<T>): T {\n');
    await handle.write(`    return visitor.visit${className}${baseName}(this);\n`);
    await handle.write('  }\n');
    await handle.write('}\n');
  }
};

const main = async (args = process.argv.slice(2)): Promise<void> => {
  if (args.length !== 1) {
    console.error('Usage: generate_ast <output directory>');
    process.exit(64);
  }

  const handle = await fs.promises.open(args[0]!, fs.constants.W_OK);

  try {
    await defineAst(handle, 'Expr', {
      Binary: {
        left: 'Expr',
        operator: 'Token',
        right: 'Expr',
      },
      Grouping: {
        expression: 'Expr',
      },
      Literal: {
        value: '{ toString(): string } | null',
      },
      Unary: {
        operator: 'Token',
        right: 'Expr',
      },
      Comma: {
        exprs: 'Expr[]',
      },
    });
  } finally {
    await handle.close();
  }
};

// eslint-disable-next-line @typescript-eslint/no-floating-promises
main();
