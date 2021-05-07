import * as path from "https://deno.land/std@0.95.0/path/mod.ts";

const keys = Object.keys as <T>(obj: T) => (keyof T)[];
const entries = Object.entries as <T>(obj: T) => [keyof T, T[keyof T]][];

const astDefinitions = {
  Expr: {
    Assign: {
      name: "Token",
      value: "Expr",
    },
    Binary: {
      left: "Expr",
      operator: "Token",
      right: "Expr",
    },
    Comma: {
      exprs: "Expr[]",
    },
    Grouping: {
      expression: "Expr",
    },
    Literal: {
      value: "{ toString(): string } | null",
    },
    Ternary: {
      condition: "Expr",
      exprIfTrue: "Expr",
      exprIfFalse: "Expr",
    },
    Unary: {
      operator: "Token",
      right: "Expr",
    },
    Variable: {
      name: "Token",
    },
    imports: ["Token"],
  },
  Stmt: {
    Block: {
      statements: "Stmt[]",
    },
    Expression: {
      expression: "Expr",
    },
    Print: {
      expression: "Expr",
    },
    Var: {
      name: "Token",
      initializer: "Expr | null",
    },
    imports: ["Expr", "Token"],
  },
} as const;

const defineAst = async (
  write: (text: string) => Promise<number>,
  baseName: keyof typeof astDefinitions,
): Promise<void> => {
  const { imports, ...classes } = astDefinitions[baseName];
  for (const im of imports) {
    await write(`import ${im} from './${im}.ts';\n`);
  }
  await write(`\nabstract class ${baseName} {
  abstract accept<T>(visitor: ${baseName}.Visitor<T>): T;
}\n`);

  await write(`\nnamespace ${baseName} {\n`);
  await write("  export interface Visitor<T> {\n");

  for (const className of keys(classes)) {
    await write(
      `    visit${className}${baseName}(${baseName.toLowerCase()}: ${className}): T;\n`,
    );
  }

  await write("  }\n");

  for (const [className, args] of entries(classes)) {
    await write(`\n  export class ${className} extends ${baseName} {
    constructor(\n`);
    for (const [name, type] of entries(args)) {
      await write(`      readonly ${String(name)}: ${type},\n`);
    }
    await write("    ) {\n");
    await write("      super();\n");
    await write("    }\n");
    await write(
      `\n    accept<T>(visitor: ${baseName}.Visitor<T>): T {\n`,
    );
    await write(
      `      return visitor.visit${className}${baseName}(this);\n`,
    );
    await write("    }\n");
    await write("  }\n");
  }
  await write("}\n");
  await write(`\nexport default ${baseName};\n`);
};

const main = async (args = Deno.args): Promise<void> => {
  const outputDir = args[0];
  if (!outputDir) {
    console.error("Usage: generate_ast <output directory>");
    Deno.exit(64);
  }

  for (const name of keys(astDefinitions)) {
    const handle = await Deno.open(path.resolve(outputDir, `${name}.ts`));
    const encoder = new TextEncoder();
    const write = (string: string) => handle.write(encoder.encode(string));
    try {
      await defineAst(write, name);
    } finally {
      handle.close();
    }
  }
};

// eslint-disable-next-line @typescript-eslint/no-floating-promises
main();
