/**
 * Custom error class for API requests that preserves HTTP status codes
 */
export class ApiError extends Error {
  public fieldErrors?: Record<string, string>;

  constructor(
    public status: number,
    public statusText: string,
    message?: string,
    fieldErrors?: Record<string, string>,
  ) {
    super(message || `API error: ${status} ${statusText}`);
    this.name = "ApiError";
    this.fieldErrors = fieldErrors;
  }

  /**
   * Build an ApiError from a failed fetch Response, reading the JSON body to
   * extract `error`, `code`, and `fieldErrors` if present.
   */
  static async fromResponse(response: Response): Promise<ApiError> {
    let message: string | undefined;
    let fieldErrors: Record<string, string> | undefined;

    try {
      const body = await response.json();
      if (body && typeof body === "object") {
        if (typeof body.error === "string") {
          message = body.error;
        }
        if (
          body.fieldErrors &&
          typeof body.fieldErrors === "object" &&
          !Array.isArray(body.fieldErrors)
        ) {
          fieldErrors = body.fieldErrors as Record<string, string>;
        }
      }
    } catch {
      // Body is not JSON or is empty — fall back to statusText
    }

    return new ApiError(
      response.status,
      response.statusText,
      message,
      fieldErrors,
    );
  }

  /**
   * Check if an error is a 404 Not Found error
   */
  static isNotFound(error: unknown): boolean {
    return error instanceof ApiError && error.status === 404;
  }

  /**
   * Check if an error is an authentication error (401/403)
   */
  static isAuthError(error: unknown): boolean {
    return (
      error instanceof ApiError &&
      (error.status === 401 || error.status === 403)
    );
  }

  /**
   * Check if an error is a server error (5xx)
   */
  static isServerError(error: unknown): boolean {
    return (
      error instanceof ApiError && error.status >= 500 && error.status < 600
    );
  }

  static network(cause: unknown): ApiError {
    const message = cause instanceof Error ? cause.message : String(cause);
    return new ApiError(0, "Network Error", message);
  }
}
