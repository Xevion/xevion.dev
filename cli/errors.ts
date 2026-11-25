export type CliErrorKind =
  | "config"
  | "connect"
  | "http"
  | "unauthorized"
  | "decode"
  | "io"
  | "json"
  | "invalid"
  | "approval"
  | "internal";

interface CliErrorInit {
  code?: string;
  help?: string;
  status?: number;
  fieldErrors?: Record<string, string>;
  retryAfter?: number;
  cause?: unknown;
}

/** Every failure the CLI reports; carries what the human and JSON renderers need. */
export class CliError extends Error {
  readonly code?: string;
  readonly help?: string;
  readonly status?: number;
  readonly fieldErrors?: Record<string, string>;
  /** Seconds the server asked the client to wait, from a 429 Retry-After. */
  readonly retryAfter?: number;

  constructor(
    readonly kind: CliErrorKind,
    message: string,
    init: CliErrorInit = {},
  ) {
    super(
      message,
      init.cause === undefined ? undefined : { cause: init.cause },
    );
    this.name = "CliError";
    this.code = init.code;
    this.help = init.help;
    this.status = init.status;
    this.fieldErrors = init.fieldErrors;
    this.retryAfter = init.retryAfter;
  }

  static invalid(message: string): CliError {
    return new CliError("invalid", message, { code: "xevion::cli::invalid" });
  }

  static config(message: string, help?: string): CliError {
    return new CliError("config", message, {
      code: "xevion::cli::config",
      help,
    });
  }

  static connect(url: string, cause: unknown): CliError {
    const detail = cause instanceof Error ? cause.message : String(cause);
    return new CliError("connect", `could not connect to ${url}: ${detail}`, {
      code: "xevion::cli::connect",
      help: "is the server running and reachable? check the target URL with `xevion targets`",
      cause,
    });
  }

  static unauthorized(): CliError {
    return new CliError("unauthorized", "not authorized for this request", {
      code: "xevion::cli::unauthorized",
      help: "authorize this target first: xevion login",
      status: 401,
    });
  }

  static http(
    status: number,
    message: string,
    init: Pick<CliErrorInit, "code" | "fieldErrors" | "retryAfter"> = {},
  ): CliError {
    return new CliError("http", `server returned ${status}: ${message}`, {
      ...init,
      status,
    });
  }

  static decode(cause: unknown): CliError {
    return new CliError("decode", "could not decode the server response", {
      code: "xevion::cli::decode",
      cause,
    });
  }

  static io(path: string, cause: unknown): CliError {
    const detail = cause instanceof Error ? cause.message : String(cause);
    return new CliError("io", `could not read ${path}: ${detail}`, {
      code: "xevion::cli::io",
      cause,
    });
  }

  static json(path: string, cause: unknown): CliError {
    const detail = cause instanceof Error ? cause.message : String(cause);
    return new CliError("json", `${path} is not valid JSON: ${detail}`, {
      code: "xevion::cli::json",
      cause,
    });
  }

  static approval(reason: string): CliError {
    return new CliError("approval", `authorization ${reason}`, {
      code: "xevion::cli::approval",
      help: "re-run `xevion login` and approve the request in the browser",
    });
  }

  /** Process exit code, following sysexits conventions where they fit. */
  get exitCode(): number {
    switch (this.kind) {
      case "connect":
        return 69;
      case "unauthorized":
        return 77;
      case "invalid":
        return 65;
      case "io":
        return 66;
      default:
        return 1;
    }
  }

  /** Machine-readable error object printed on stdout under `--json`. */
  toJSON(): Record<string, unknown> {
    return {
      error: this.message,
      code: this.code ?? null,
      help: this.help ?? null,
      ...(this.status === undefined ? {} : { status: this.status }),
      ...(this.fieldErrors ? { fieldErrors: this.fieldErrors } : {}),
    };
  }
}

export function toCliError(e: unknown): CliError {
  if (e instanceof CliError) return e;
  const message = e instanceof Error ? e.message : String(e);
  return new CliError("internal", message, { cause: e });
}
