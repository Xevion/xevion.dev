import { spawn } from "node:child_process";
import { hostname } from "node:os";
import ansis from "ansis";
import { ApiClient } from "./client.ts";
import { Config, resolveConfigPath } from "./config.ts";
import { CliError } from "./errors.ts";
import type { CommandNode, Context } from "./framework.ts";
import * as output from "./output.ts";
import type { ApiSession } from "./types.ts";

interface DeviceStart {
  id: string;
  userCode: string;
  verificationUrl: string;
  expiresAt: string;
  interval: number;
}

type DevicePoll =
  | { status: "pending" }
  | { status: "approved"; token: string; username?: string }
  | { status: "denied" };

const DEFAULT_TTL_MS = 10 * 60 * 1000;

export function loadConfig(ctx: Context): Config {
  return Config.load(resolveConfigPath(ctx.config));
}

/** Resolve the selected target and build an authenticated client. */
export function authedClient(config: Config, api?: string): ApiClient {
  const { entry } = config.resolve(api);
  return new ApiClient(entry.url, entry.token);
}

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

/** Open a URL with the platform opener; resolves once the opener is launched or has failed. */
function openBrowser(url: string): Promise<void> {
  const [cmd, args]: [string, string[]] =
    process.platform === "darwin"
      ? ["open", [url]]
      : process.platform === "win32"
        ? ["cmd", ["/c", "start", "", url]]
        : ["xdg-open", [url]];
  return new Promise((resolve, reject) => {
    const child = spawn(cmd, args, { stdio: "ignore", detached: true });
    child.once("error", reject);
    child.once("spawn", () => {
      child.unref();
      resolve();
    });
  });
}

function resolveVerificationUrl(
  baseUrl: string,
  verificationUrl: string,
): string {
  return verificationUrl.startsWith("/")
    ? `${baseUrl}${verificationUrl}`
    : verificationUrl;
}

async function pollForApproval(
  client: ApiClient,
  start: DeviceStart,
): Promise<DevicePoll & { status: "approved" | "denied" }> {
  const parsed = Date.parse(start.expiresAt);
  const deadline = Number.isFinite(parsed)
    ? parsed
    : Date.now() + DEFAULT_TTL_MS;
  const interval = Math.max(1, start.interval || 5) * 1000;
  let wait = interval;

  while (Date.now() < deadline) {
    await sleep(wait);
    wait = interval;
    let poll: DevicePoll;
    try {
      poll = await client.get<DevicePoll>(
        `/api/auth/device/${encodeURIComponent(start.id)}`,
      );
    } catch (e) {
      if (e instanceof CliError && e.status === 404) {
        throw CliError.approval("request expired before approval");
      }
      if (e instanceof CliError && e.status === 429) {
        wait = Math.max(interval, (e.retryAfter ?? 0) * 1000);
        continue;
      }
      throw e;
    }
    if (poll.status === "approved" || poll.status === "denied") return poll;
  }
  throw CliError.approval("request expired before approval");
}

async function login(
  flags: { url?: string; label?: string; noBrowser?: boolean },
  ctx: Context,
): Promise<void> {
  const config = loadConfig(ctx);

  // No magic names: a first-time user must say which target they are authorizing.
  const name = ctx.api ?? config.default;
  if (name === undefined) {
    throw CliError.invalid(
      "no target selected: pass --api <name> (and --url <url> for a new target)",
    );
  }
  const url = flags.url ?? config.urlFor(name);
  if (url === undefined) {
    throw CliError.invalid(
      `no URL known for target '${name}': pass --url <url> to authorize it`,
    );
  }
  const baseUrl = url.replace(/\/+$/, "");
  const label = flags.label ?? `xevion CLI on ${hostname()}`;

  const client = new ApiClient(baseUrl);
  const start = await client.post<DeviceStart>("/api/auth/device", { label });
  const approvalUrl = resolveVerificationUrl(baseUrl, start.verificationUrl);

  output.info(
    `Confirm this code in your browser: ${ansis.bold(start.userCode)}`,
  );
  output.info(`Approval URL: ${approvalUrl}`);
  output.info("Type the code on that page to approve this device.");

  if (flags.noBrowser) {
    output.info("Open the URL above to approve.");
  } else {
    try {
      await openBrowser(approvalUrl);
    } catch (e) {
      output.error(
        `Couldn't open a browser (${e instanceof Error ? e.message : String(e)}); open the URL above manually.`,
      );
    }
  }

  output.info("Waiting for approval…");
  const result = await pollForApproval(client, start);
  if (result.status === "denied") {
    throw CliError.approval("was denied in the browser");
  }

  config.setTarget(name, baseUrl, result.token);
  config.save();

  let username = result.username;
  if (username === undefined) {
    try {
      username = (
        await new ApiClient(baseUrl, result.token).get<ApiSession>(
          "/api/session",
        )
      ).username;
    } catch {
      username = undefined;
    }
  }

  if (ctx.json) {
    output.printJson({
      target: name,
      url: baseUrl,
      username: username ?? null,
      authenticated: true,
    });
  } else {
    output.success(
      `Authorized${username ? ` as ${ansis.bold(username)}` : ""} for target '${name}'`,
    );
    output.info(`Saved to ${config.path}`);
  }
}

async function logout(ctx: Context): Promise<void> {
  const config = loadConfig(ctx);
  const { name, entry } = config.resolve(ctx.api);
  // Best-effort server-side revoke; the local token is cleared regardless.
  try {
    await new ApiClient(entry.url, entry.token).post("/api/logout");
  } catch {
    // ignored
  }
  config.clearToken(name);
  config.save();
  if (ctx.json) {
    output.printJson({ target: name, authenticated: false });
  } else {
    output.success(`Logged out of target '${name}'`);
  }
}

async function session(ctx: Context): Promise<void> {
  const config = loadConfig(ctx);
  const { name, entry } = config.resolve(ctx.api);
  const client = new ApiClient(entry.url, entry.token);
  const s = await client.get<ApiSession>("/api/session");

  if (ctx.json) {
    output.printJson({
      authenticated: s.authenticated,
      username: s.username,
      sessionType: s.sessionType,
      expiresAt: s.expiresAt,
      target: name,
      apiUrl: client.apiUrl,
    });
  } else if (s.authenticated) {
    output.printSession(s.username, client.apiUrl);
    output.stderr(`  ${ansis.dim("Target:")} ${name}`);
    output.stderr(`  ${ansis.dim("Type:")} ${s.sessionType}`);
    output.stderr(`  ${ansis.dim("Expires:")} ${s.expiresAt}`);
  } else {
    output.error("Session expired or invalid");
  }
}

/** Report a target mutation: a stable machine action under --json, a sentence otherwise. */
function confirmTarget(
  action: "default" | "added" | "removed" | "renamed",
  name: string,
  config: Config,
  ctx: Context,
): void {
  if (ctx.json) {
    output.printJson({
      action,
      target: name,
      default: config.default === name,
    });
    return;
  }
  const human = {
    default: `Target '${name}' is now the default`,
    added: `Target '${name}' added`,
    removed: `Target '${name}' removed`,
    renamed: `Renamed target to '${name}'`,
  }[action];
  output.success(human);
}

function listTargets(config: Config, ctx: Context): void {
  if (ctx.json) {
    output.printJson({
      targets: config.names().map((name) => ({
        name,
        url: config.urlFor(name) ?? null,
        default: config.default === name,
        authorized: config.isAuthorized(name),
      })),
      path: config.path,
    });
    return;
  }
  if (config.names().length === 0) {
    output.info(
      "No API targets configured. Add one: xevion targets add <name> --url <url>",
    );
    return;
  }
  for (const name of config.names()) {
    const marker = config.default === name ? "*" : " ";
    const auth = config.isAuthorized(name)
      ? ansis.green("authorized")
      : ansis.dim("no token");
    console.log(
      `${marker} ${ansis.bold(name)}  ${ansis.dim(config.urlFor(name) ?? "")}  ${auth}`,
    );
  }
  output.info(`config: ${config.path}`);
}

function stringParam(
  value: string | string[] | undefined,
  name: string,
): string {
  if (typeof value !== "string") {
    throw CliError.invalid(`missing required argument <${name}>`);
  }
  return value;
}

export const authCommands: CommandNode[] = [
  {
    name: "login",
    description:
      "Authorize this CLI via the browser and save a long-lived token",
    flags: {
      url: {
        type: String,
        description:
          "API base URL (required when first authorizing a new target)",
        placeholder: "<url>",
      },
      label: {
        type: String,
        description:
          "Device label shown in the approval dialog (defaults to `xevion CLI on <hostname>`)",
        placeholder: "<label>",
      },
      noBrowser: {
        type: Boolean,
        description: "Print the approval URL instead of opening a browser",
      },
    },
    run: ({ flags }, ctx) => login(flags, ctx),
  },
  {
    name: "logout",
    description: "Revoke and clear the saved token for the selected target",
    run: (_p, ctx) => logout(ctx),
  },
  {
    name: "session",
    description: "Check current session status",
    run: (_p, ctx) => session(ctx),
  },
  {
    name: "targets",
    description: "List and manage configured API targets",
    run: (_p, ctx) => listTargets(loadConfig(ctx), ctx),
    children: [
      {
        name: "use",
        description: "Make a target the default (used when --api is omitted)",
        parameters: ["<name>"],
        run: ({ _ }, ctx) => {
          const name = stringParam(_.name, "name");
          const config = loadConfig(ctx);
          config.setDefault(name);
          config.save();
          confirmTarget("default", name, config, ctx);
        },
      },
      {
        name: "add",
        description: "Add a target without authorizing it (set its base URL)",
        parameters: ["<name>"],
        flags: {
          url: {
            type: String,
            description: "API base URL",
            placeholder: "<url>",
          },
        },
        run: ({ _, flags }, ctx) => {
          const name = stringParam(_.name, "name");
          if (typeof flags.url !== "string") {
            throw CliError.invalid(
              "the following required arguments were not provided: --url <url>",
            );
          }
          const config = loadConfig(ctx);
          config.setTarget(name, flags.url);
          config.save();
          confirmTarget("added", name, config, ctx);
        },
      },
      {
        name: "rm",
        description: "Remove a target",
        parameters: ["<name>"],
        run: ({ _ }, ctx) => {
          const name = stringParam(_.name, "name");
          const config = loadConfig(ctx);
          const entry = config.removeTarget(name);
          config.save();
          if (!ctx.json && entry.token !== undefined) {
            output.info(
              "Removed an authorized target; its server-side token was not revoked (use `logout` first to revoke).",
            );
          }
          confirmTarget("removed", name, config, ctx);
        },
      },
      {
        name: "rename",
        description:
          "Rename a target, preserving its URL, token, and default status",
        parameters: ["<from>", "<to>"],
        run: ({ _ }, ctx) => {
          const from = stringParam(_.from, "from");
          const to = stringParam(_.to, "to");
          const config = loadConfig(ctx);
          config.renameTarget(from, to);
          config.save();
          confirmTarget("renamed", to, config, ctx);
        },
      },
    ],
  },
  {
    name: "config",
    description: "Inspect CLI configuration",
    children: [
      {
        name: "path",
        description: "Print the resolved config file path",
        run: (_p, ctx) => {
          const path = loadConfig(ctx).path;
          console.log(ctx.json ? JSON.stringify({ path }) : path);
        },
      },
    ],
  },
];
