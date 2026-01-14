/**
 * Custom error class for API requests that preserves HTTP status codes
 */
export class ApiError extends Error {
  constructor(
    public status: number,
    public statusText: string,
    message?: string,
  ) {
    super(message || `API error: ${status} ${statusText}`);
    this.name = "ApiError";
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
    return error instanceof ApiError && error.status >= 500 && error.status < 600;
  }
}
