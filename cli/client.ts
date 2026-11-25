import { CliError } from "./errors.ts";

const REQUEST_TIMEOUT_MS = 30_000;

export interface SendOptions {
  body?: unknown;
  headers?: Record<string, string>;
}

export interface ApiResult<T> {
  data: T;
  headers: Headers;
  status: number;
}

/** API client that authenticates with a long-lived CLI bearer token. */
export class ApiClient {
  readonly apiUrl: string;

  constructor(
    apiUrl: string,
    private readonly token?: string,
  ) {
    this.apiUrl = apiUrl.replace(/\/+$/, "");
  }

  /**
   * Send a request and unwrap the `{ data }` envelope. Non-2xx responses throw a
   * CliError built from the server's `{ message, code, fieldErrors }` body.
   */
  async send<T = unknown>(
    method: string,
    path: string,
    options: SendOptions = {},
  ): Promise<ApiResult<T>> {
    const url = `${this.apiUrl}${path}`;
    const headers: Record<string, string> = {
      accept: "application/json",
      ...options.headers,
    };
    if (this.token) headers.authorization = `Bearer ${this.token}`;
    if (options.body !== undefined)
      headers["content-type"] = "application/json";

    let response: Response;
    try {
      response = await fetch(url, {
        method,
        headers,
        body:
          options.body === undefined ? undefined : JSON.stringify(options.body),
        signal: AbortSignal.timeout(REQUEST_TIMEOUT_MS),
      });
    } catch (e) {
      throw CliError.connect(url, e);
    }

    const text = await response.text().catch(() => "");
    if (!response.ok) throw httpError(response, text);

    if (text.trim() === "") {
      return {
        data: undefined as T,
        headers: response.headers,
        status: response.status,
      };
    }
    let body: unknown;
    try {
      body = JSON.parse(text);
    } catch (e) {
      throw CliError.decode(e);
    }
    const data =
      typeof body === "object" && body !== null && "data" in body
        ? (body as { data: T }).data
        : (body as T);
    return { data, headers: response.headers, status: response.status };
  }

  async get<T = unknown>(path: string): Promise<T> {
    return (await this.send<T>("GET", path)).data;
  }

  async post<T = unknown>(path: string, body: unknown = {}): Promise<T> {
    return (await this.send<T>("POST", path, { body })).data;
  }

  async put<T = unknown>(path: string, body: unknown): Promise<T> {
    return (await this.send<T>("PUT", path, { body })).data;
  }

  async delete<T = unknown>(path: string): Promise<T> {
    return (await this.send<T>("DELETE", path)).data;
  }
}

interface ErrorBody {
  message?: unknown;
  error?: unknown;
  code?: unknown;
  fieldErrors?: unknown;
}

function httpError(response: Response, text: string): CliError {
  const status = response.status;
  if (status === 401) return CliError.unauthorized();

  let body: ErrorBody | undefined;
  try {
    const parsed: unknown = JSON.parse(text);
    if (typeof parsed === "object" && parsed !== null)
      body = parsed as ErrorBody;
  } catch {
    body = undefined;
  }

  const raw = body?.message ?? body?.error;
  const message =
    typeof raw === "string" && raw !== ""
      ? raw
      : text.trim() === ""
        ? "(empty response body)"
        : text;
  const code = typeof body?.code === "string" ? body.code : undefined;
  const fieldErrors =
    typeof body?.fieldErrors === "object" &&
    body.fieldErrors !== null &&
    !Array.isArray(body.fieldErrors)
      ? (body.fieldErrors as Record<string, string>)
      : undefined;

  const retry = Number(response.headers.get("retry-after"));
  const retryAfter = Number.isFinite(retry) && retry > 0 ? retry : undefined;
  return CliError.http(status, message, { code, fieldErrors, retryAfter });
}
