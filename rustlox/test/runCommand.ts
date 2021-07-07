const textDecoder = new TextDecoder();

interface RunResult {
  code: number;
  stdout: string;
  stderr: string;
}

const runCommand = async (
  ...cmd: string[]
): Promise<RunResult> => {
  const p = Deno.run({
    cmd,
    stdout: "piped",
    stderr: "piped",
  });

  const [{ code }, rawOutput, rawError] = await Promise.all([
    p.status(),
    p.output(),
    p.stderrOutput(),
  ]);

  p.close();

  return {
    code,
    stdout: textDecoder.decode(rawOutput).trim(),
    stderr: textDecoder.decode(rawError),
  };
};

export default runCommand;
