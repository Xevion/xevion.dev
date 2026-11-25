import { CliError, toCliError } from "./errors.ts";
import { UsageExit, dispatch } from "./framework.ts";
import { authCommands } from "./auth.ts";
import { printCliError } from "./output.ts";
import { projectsCommand } from "./projects.ts";
import { settingsCommand } from "./settings.ts";
import { tagsCommand } from "./tags.ts";

const root = {
  name: "xevion",
  description: "Manage xevion.dev content remotely (projects, tags, settings)",
  children: [...authCommands, projectsCommand, tagsCommand, settingsCommand],
};

async function main(): Promise<void> {
  const argv = process.argv.slice(2);
  const json = argv.includes("--json");
  try {
    await dispatch(root, argv);
  } catch (e) {
    if (e instanceof UsageExit) {
      if (e.toStdout) console.log(e.output);
      else process.stderr.write(`${e.output}\n`);
      process.exitCode = e.exitCode;
      return;
    }
    const err: CliError = toCliError(e);
    if (json) {
      // Machine-readable error on stdout for scripted callers.
      console.log(JSON.stringify(err.toJSON()));
    } else {
      printCliError(err);
    }
    process.exitCode = err.exitCode;
  }
}

await main();
