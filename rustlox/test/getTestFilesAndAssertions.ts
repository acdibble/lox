import * as path from "https://deno.land/std@0.99.0/path/mod.ts";

type FileExpectation =
  | { code: 0 | 65; expectations: string[] }
  | { code: 70; error: string };

const parseTestFile = async (filename: string): Promise<FileExpectation> => {
  const file = await Deno.readTextFile(filename);

  const runtimeError = /\/\/ expect runtime error: (.+)/.exec(file)?.[1];

  if (runtimeError) {
    return { code: 70, error: runtimeError };
  }

  const parseErrorRegExp = /\/\/ \[/g;
  const expectRegExp = /\/\/ expect: (.+)/g;

  const expectations: string[] = [];
  let code: 0 | 65 = 0;

  let re: RegExp = expectRegExp;

  if (parseErrorRegExp.test(file)) {
    code = 65;
    re = parseErrorRegExp;
  }

  for (const match of file.matchAll(re)) {
    expectations.push(match[1]);
  }

  return { code, expectations };
};

const getLoxFiles = async (dirname: string): Promise<string[]> => {
  const files: string[] = [];
  const dirs: string[] = [];

  for await (const entry of Deno.readDir(dirname)) {
    if (
      entry.isFile && Deno.args.length
        ? Deno.args.includes(entry.name)
        : entry.name.endsWith(".lox")
    ) {
      files.push(path.join(dirname, entry.name));
    } else if (entry.isDirectory) {
      dirs.push(path.join(dirname, entry.name));
    }
  }

  return files.concat(
    ...(await Promise.all(dirs.map((dir) => getLoxFiles(dir)))),
  );
};

const getTestFilesAndAssertions = async (): Promise<
  Record<string, FileExpectation>
> => {
  const files = await getLoxFiles("test");
  const expectations = await Promise.all(files.map(parseTestFile));

  return files.reduce((acc, filename, i) => {
    acc[filename] = expectations[i];
    return acc;
  }, {} as Record<string, FileExpectation>);
};

export default getTestFilesAndAssertions;
