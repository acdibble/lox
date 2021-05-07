import * as fs from 'fs';
import * as path from 'path';

const keys = Object.keys as <T>(obj: T) => (keyof T)[];
const entries = Object.entries as <T>(obj: T) => [keyof T, T[keyof T]][];

const astDefinitions = {
  Expr: {
    Assign: {
      name: 'Token',
      value: 'Expr',
    },
    Binary: {
      left: 'Expr',
      operator: 'Token',
      right: 'Expr',
    },
    Comma: {
      exprs: 'Expr[]',
    },
    Grouping: {
      expression: 'Expr',
    },
    Literal: {
      value: '{ toString(): string } | null',
    },
    Ternary: {
      condition: 'Expr',
      exprIfTrue: 'Expr',
      exprIfFalse: 'Expr',
    },
    Unary: {
      operator: 'Token',
      right: 'Expr',
    },
    Variable: {
      name: 'Token',
    },
    imports: ['Token'],
  },
  Stmt: {
    Expression: {
      expression: 'Expr',
    },
    Print: {
      expression: 'Expr',
    },
    Var: {
      name: 'Token',
      initializer: 'Expr | null',
    },
    imports: ['Expr', 'Token'],
  },
} as const;

const defineAst = async (handle: fs.promises.FileHandle, baseName: keyof typeof astDefinitions): Promise<void> => {
  const { imports, ...classes } = astDefinitions[baseName];
  for (const im of imports) {
    await handle.write(`import ${im} from './${im}.js';\n`);
  }
  await handle.write(`\nabstract class ${baseName} {
  abstract accept<T>(visitor: ${baseName}.Visitor<T>): T;
}\n`);

  await handle.write(`\nnamespace ${baseName} {\n`);
  await handle.write('  export interface Visitor<T> {\n');

  for (const className of keys(classes)) {
    await handle.write(`    visit${className}${baseName}(${baseName.toLowerCase()}: ${className}): T;\n`);
  }

  await handle.write('  }\n');

  for (const [className, args] of entries(classes)) {
    await handle.write(`\n  export class ${className} extends ${baseName} {
    constructor(\n`);
    for (const [name, type] of entries(args)) {
      await handle.write(`      readonly ${String(name)}: ${type},\n`);
    }
    await handle.write('    ) {\n');
    await handle.write('      super();\n');
    await handle.write('    }\n');
    await handle.write(`\n    accept<T>(visitor: ${baseName}.Visitor<T>): T {\n`);
    await handle.write(`      return visitor.visit${className}${baseName}(this);\n`);
    await handle.write('    }\n');
    await handle.write('  }\n');
  }
  await handle.write('}\n');
  await handle.write(`\nexport default ${baseName};\n`);
};

const main = async (args = process.argv.slice(2)): Promise<void> => {
  const outputDir = args[0];
  if (!outputDir) {
    console.error('Usage: generate_ast <output directory>');
    process.exit(64);
  }

  for (const name of keys(astDefinitions)) {
    const handle = await fs.promises.open(path.resolve(outputDir, `${name}.ts`), 'w+');
    try {
      await defineAst(handle, name);
    } finally {
      await handle.close();
    }
  }
};

// eslint-disable-next-line @typescript-eslint/no-floating-promises
main();
