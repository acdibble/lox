import * as fs from 'fs';

const keys = Object.keys as <T>(obj: T) => (keyof T)[];
const entries = Object.entries as <T>(obj: T) => [keyof T, T[keyof T]][];

const defineAst = (baseName: string, classes: Record<string, Record<string, string>>): string => {
  const header = `import Token from './Token.js';

export abstract class ${baseName} {
  abstract accept<T>(visitor: Visitor<T>): T;
}`;

  // eslint-disable-next-line arrow-body-style
  const visitor = `export interface Visitor<T> {\n  ${keys(classes).map((className) => {
    return `visit${className}${baseName}(${baseName.toLowerCase()}: ${className}): T;`;
  }).join('\n  ')}\n}`;

  const generatedClasses = entries(classes).map(([className, args]) => {
    let classText = `export class ${className} extends Expr {
  constructor(\n`;
    for (const [name, type] of entries(args)) {
      classText += `    readonly ${name}: ${type},\n`;
    }
    classText += '  ) {\n';
    classText += '    super();\n';
    classText += '  }\n';
    classText += '\n  accept<T>(visitor: Visitor<T>): T {\n';
    classText += `    return visitor.visit${className}${baseName}(this);\n`;
    classText += '  }\n';
    classText += '}';

    return classText;
  });

  return `${[header, visitor, generatedClasses.join('\n\n')].join('\n\n')}\n`;
};

const main = async (args = process.argv.slice(2)): Promise<void> => {
  if (args.length !== 1) {
    console.error('Usage: generate_ast <output directory>');
    process.exit(64);
  }

  const ast = defineAst('Expr', {
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
  });

  await fs.promises.writeFile(args[0]!, ast, 'utf8');
};

// eslint-disable-next-line @typescript-eslint/no-floating-promises
main();
