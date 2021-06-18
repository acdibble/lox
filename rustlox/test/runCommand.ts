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

  const { code } = await p.status();

  // Reading the outputs closes their pipes
  const rawOutput = await p.output();
  const rawError = await p.stderrOutput();
  p.close();

  return {
    code,
    stdout: textDecoder.decode(rawOutput).trim(),
    stderr: textDecoder.decode(rawError),
  };
};

export default runCommand;
