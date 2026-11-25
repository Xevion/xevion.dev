export interface ApiError {
  status: number;
  code: string;
  message: string;
  fieldErrors?: Record<string, string>;
}

export function isNotFound(error: ApiError): boolean {
  return error.status === 404;
}

export function isAuthError(error: ApiError): boolean {
  return error.status === 401 || error.status === 403;
}

export function toApiError(status: number, body: unknown): ApiError {
  const record =
    body && typeof body === "object" ? (body as Record<string, unknown>) : {};
  const fields = record.fieldErrors;
  return {
    status,
    code: typeof record.code === "string" ? record.code : "UNKNOWN",
    message:
      typeof record.message === "string"
        ? record.message
        : `request failed (${status})`,
    ...(fields && typeof fields === "object" && !Array.isArray(fields)
      ? { fieldErrors: fields as Record<string, string> }
      : {}),
  };
}

export const NETWORK_ERROR: ApiError = {
  status: 0,
  code: "NETWORK_ERROR",
  message: "Network request failed",
};
