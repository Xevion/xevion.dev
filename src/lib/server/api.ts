import { error } from "@sveltejs/kit";
import { type, type Type, type distill } from "arktype";

/** Success envelope shared by every /api/* endpoint. */
export function ok<T>(data: T, init?: ResponseInit): Response {
  return Response.json({ data }, init);
}

export type ApiErrorCode =
  | "VALIDATION_ERROR"
  | "BAD_REQUEST"
  | "UNAUTHORIZED"
  | "FORBIDDEN"
  | "NOT_FOUND"
  | "METHOD_NOT_ALLOWED"
  | "CONFLICT"
  | "PRECONDITION_FAILED"
  | "UNSUPPORTED_MEDIA_TYPE"
  | "RATE_LIMITED"
  | "BAD_GATEWAY"
  | "SERVICE_UNAVAILABLE"
  | "INTERNAL";

const STATUS_CODES: Record<number, ApiErrorCode> = {
  400: "VALIDATION_ERROR",
  401: "UNAUTHORIZED",
  403: "FORBIDDEN",
  404: "NOT_FOUND",
  405: "METHOD_NOT_ALLOWED",
  409: "CONFLICT",
  412: "PRECONDITION_FAILED",
  415: "UNSUPPORTED_MEDIA_TYPE",
  429: "RATE_LIMITED",
  503: "SERVICE_UNAVAILABLE",
};

/** Throws a SvelteKit error carrying a machine-readable code (and optional per-field messages). */
export function apiError(
  status: number,
  code: ApiErrorCode,
  message: string,
  fieldErrors?: Record<string, string>,
): never {
  error(status, message, { code, ...(fieldErrors ? { fieldErrors } : {}) });
}

export const notFound = (what: string): never =>
  apiError(404, "NOT_FOUND", `${what} not found`);

// A missing method comes back as a returned text/plain response, not a thrown error.
export async function jsonifyErrorResponse(
  response: Response,
): Promise<Response> {
  if (
    response.status < 400 ||
    response.headers.get("content-type")?.includes("json")
  ) {
    return response;
  }
  const isText =
    response.headers.get("content-type")?.startsWith("text/plain") ?? false;
  const body = isText ? (await response.text()).trim() : "";
  const headers = new Headers(response.headers);
  headers.delete("content-type");
  headers.delete("content-length");
  headers.delete("content-encoding");
  const code =
    STATUS_CODES[response.status] ??
    (response.status >= 500 ? "INTERNAL" : "BAD_REQUEST");
  return Response.json(
    {
      message:
        body || response.statusText || `request failed (${response.status})`,
      code,
    },
    { status: response.status, headers },
  );
}

/** Parses and validates a JSON body, failing the request with a 400 on mismatch. */
export async function parseJsonBody<t>(
  request: Request,
  schema: Type<t>,
): Promise<distill.Out<t>> {
  const raw = await request.json().catch(() => undefined);
  const result = schema(raw);
  if (result instanceof type.errors)
    apiError(400, "VALIDATION_ERROR", result.summary);
  return result as never;
}
