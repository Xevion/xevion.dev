import { cli, type Flags } from "cleye";
import { CliError } from "./errors.ts";

export interface Context {
  api?: string;
  config?: string;
  json: boolean;
}

export interface Parsed {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any -- flag types vary per command
  flags: Record<string, any>;
  _: Record<string, string | string[] | undefined>;
}

export interface CommandNode {
  name: string;
  description: string;
  /** Positional parameters, e.g. "<ref>" or "[locator]". */
  parameters?: string[];
  flags?: Flags;
  run?: (parsed: Parsed, ctx: Context) => Promise<void> | void;
  children?: CommandNode[];
}

export const VERSION = "0.1.0";

const GLOBAL_FLAGS = {
  api: {
    type: String,
    description:
      "Named API target from the config [env: XEVION_API; default: config `default`]",
    placeholder: "<name>",
  },
  config: {
    type: String,
    description:
      "Path to the config file [env: XEVION_CONFIG; default: ~/.config/xevion/config.toml]",
    placeholder: "<file>",
  },
  json: {
    type: Boolean,
    description: "Output raw JSON instead of formatted text",
  },
} satisfies Flags;

const GLOBAL_VALUE_FLAGS = new Set(["--api", "--config"]);

function isHelp(token: string): boolean {
  return token === "--help" || token === "-h";
}

function groupHelp(path: string[], node: CommandNode): string {
  const title = ["xevion", ...path].join(" ");
  const children = node.children ?? [];
  const width = Math.max(...children.map((c) => c.name.length), 4);
  const lines = [
    `${title} - ${node.description}`,
    "",
    `Usage: ${title} <command> [flags]`,
    "",
    "Commands:",
    ...children.map((c) => `  ${c.name.padEnd(width)}  ${c.description}`),
    "",
    "Global flags:",
    "  --api <name>     Named API target from the config [env: XEVION_API]",
    "  --config <file>  Path to the config file [env: XEVION_CONFIG]",
    "  --json           Output raw JSON instead of formatted text",
    "  -h, --help       Show help",
  ];
  if (path.length === 0) lines.push("  -V, --version    Show version");
  return lines.join("\n");
}

class UsageExit extends Error {
  constructor(
    readonly exitCode: number,
    readonly output: string,
    readonly toStdout: boolean,
  ) {
    super(output);
  }
}

/**
 * Walk the leading words of argv down the command tree, skipping the global
 * value flags, and stop at the first flag. Returns the matched node path and
 * the argv with the matched command words removed.
 */
function resolve(
  root: CommandNode,
  argv: string[],
): { node: CommandNode; path: string[]; rest: string[]; stray?: string } {
  const rest: string[] = [];
  const path: string[] = [];
  let node = root;
  let stray: string | undefined;
  let descending = true;
  for (let i = 0; i < argv.length; i++) {
    const token = argv[i]!;
    if (descending && GLOBAL_VALUE_FLAGS.has(token)) {
      rest.push(token, argv[i + 1] ?? "");
      i++;
      continue;
    }
    if (descending && !token.startsWith("-")) {
      const child = node.children?.find((c) => c.name === token);
      if (child) {
        node = child;
        path.push(token);
        continue;
      }
      descending = false;
      if (node.children) stray = token;
    } else if (token.startsWith("-")) {
      descending = false;
    }
    rest.push(token);
  }
  return { node, path, rest, stray };
}

/** Parse and run one invocation. Help and usage failures surface as UsageExit. */
export async function dispatch(
  root: CommandNode,
  argv: string[],
): Promise<void> {
  if (argv.includes("--version") || (argv.length === 1 && argv[0] === "-V")) {
    throw new UsageExit(0, `xevion ${VERSION}`, true);
  }

  const { node, path, rest, stray } = resolve(root, argv);
  const bare = rest.filter((t) => !GLOBAL_VALUE_FLAGS.has(t));
  const wantsHelp = rest.some(isHelp);

  if (stray !== undefined && !wantsHelp) {
    throw new UsageExit(
      2,
      `error: unrecognized subcommand '${stray}'\n\n${groupHelp(path, node)}`,
      false,
    );
  }
  if (node.children && (wantsHelp || !node.run)) {
    const missing = !wantsHelp && bare.length === 0;
    throw new UsageExit(missing ? 2 : 0, groupHelp(path, node), !missing);
  }
  if (!node.run) {
    throw new UsageExit(2, groupHelp(path, node), false);
  }

  const parsed = cli(
    {
      name: ["xevion", ...path].join(" "),
      parameters: node.parameters,
      flags: { ...GLOBAL_FLAGS, ...node.flags },
      help: { description: node.description },
      strictFlags: true,
    },
    undefined,
    rest,
  ) as unknown as Parsed;

  const flags = parsed.flags;
  const ctx: Context = {
    api: flags.api || process.env.XEVION_API || undefined,
    config: flags.config || undefined,
    json: Boolean(flags.json),
  };
  await node.run(parsed, ctx);
}

export function requireFlag(
  value: string | undefined,
  flag: string,
  placeholder: string,
): string {
  if (value === undefined) {
    throw CliError.invalid(
      `the following required arguments were not provided: --${flag} <${placeholder}>`,
    );
  }
  return value;
}

export { UsageExit };
