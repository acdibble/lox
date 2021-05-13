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
    Call: {
      callee: "Expr",
      paren: "Token",
      args: "Expr[]",
    },
    Comma: {
      expressions: "Expr[]",
    },
    Function: {
      name: "Token | null",
      params: "Token[]",
      body: "Stmt[]",
    },
    Get: {
      object: "Expr",
      name: "Token",
    },
    Grouping: {
      expression: "Expr",
    },
    Literal: {
      value: "{ toString(): string } | null",
    },
    Logical: {
      left: "Expr",
      operator: "Token",
      right: "Expr",
    },
    Set: {
      object: "Expr",
      name: "Token",
      value: "Expr",
    },
    This: {
      keyword: "Token",
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
    imports: ["Token", "Stmt"].sort(),
  },
  Stmt: {
    Block: {
      statements: "Stmt[]",
    },
    Class: {
      name: "Token",
      methods: "Stmt.Function[]",
      classMethods: "Stmt.Function[]",
    },
    Break: {
      keyword: "Token",
    },
    Expression: {
      expression: "Expr",
    },
    Function: {
      name: "Token",
      params: "Token[]",
      body: "Stmt[]",
    },
    If: {
      condition: "Expr",
      thenBranch: "Stmt",
      elseBranch: "Stmt | null",
    },
    Print: {
      expression: "Expr",
    },
    Return: {
      keyword: "Token",
      value: "Expr | null",
    },
    Var: {
      name: "Token",
      initializer: "Expr | null",
    },
    While: {
      condition: "Expr",
      body: "Stmt",
    },
    imports: ["Expr", "Token"].sort(),
  },
} as const;

const defineAst = async (
  write: (text: string) => Promise<number>,
  baseName: keyof typeof astDefinitions,
): Promise<void> => {
  const { imports, ...classes } = astDefinitions[baseName];
  await write("// deno-lint-ignore-file no-namespace");
  for (const im of imports) {
    await write(`import type ${im} from "./${im}.ts";`);
  }
  await write(`\nabstract class ${baseName} {
  abstract accept<T>(visitor: ${baseName}.Visitor<T>): T;
}`);

  await write(`\nnamespace ${baseName} {`);
  await write("  export interface Visitor<T> {");

  for (const className of keys(classes)) {
    await write(
      `    visit${className}${baseName}(${baseName.toLowerCase()}: ${baseName}.${className}): T;`,
    );
  }

  await write("  }");

  for (const [className, args] of entries(classes)) {
    await write(`\n  export class ${className} extends ${baseName} {
    constructor(`);
    for (const [name, type] of entries(args)) {
      await write(`      readonly ${String(name)}: ${type},`);
    }
    await write("    ) {");
    await write("      super();");
    await write("    }");
    await write(
      `\n    accept<T>(visitor: ${baseName}.Visitor<T>): T {`,
    );
    await write(
      `      return visitor.visit${className}${baseName}(this);`,
    );
    await write("    }");
    await write("  }");
  }
  await write("}");
  await write(`\nexport default ${baseName};`);
};

const main = async (args = Deno.args): Promise<void> => {
  const outputDir = args[0];
  if (!outputDir) {
    console.error("Usage: generate_ast <output directory>");
    Deno.exit(64);
  }

  for (const name of keys(astDefinitions)) {
    const filename = path.resolve(outputDir, `${name}.ts`);
    const handle = await Deno.open(filename, {
      write: true,
      truncate: true,
      create: true,
    });
    const encoder = new TextEncoder();
    const write = (string: string) =>
      handle.write(encoder.encode(`${string}\n`));
    try {
      await defineAst(write, name);
    } finally {
      handle.close();
    }
  }
};

main();
