import getTestFilesAndAssertions from "./getTestFilesAndAssertions.ts";
import runCommand from "./runCommand.ts";
import { assertEquals } from "https://deno.land/std@0.99.0/testing/asserts.ts";

const filesAndAssertions = await getTestFilesAndAssertions();

await runCommand("cargo", "build", "--release");

Object.entries(filesAndAssertions).map(([filename, fileResult]) => {
  Deno.test(`rustlox ${filename}`, async () => {
    const { code, stdout, stderr } = await runCommand(
      "target/release/rustlox",
      filename,
    );
    assertEquals(code, fileResult.code);
    if (fileResult.code !== 70) {
      assertEquals(stdout.trimEnd(), fileResult.expectations.join("\n"));
    } else {
      const [actualError] = stderr.split("\n");
      assertEquals(actualError, fileResult.error);
    }
  });
});
