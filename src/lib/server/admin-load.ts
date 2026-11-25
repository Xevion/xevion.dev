import { error } from "@sveltejs/kit";

/** Fetches an in-process API route for a load function, unwrapping `{ data }` or throwing the API's error. */
export async function loadApi<T>(
  fetch: typeof globalThis.fetch,
  path: string,
): Promise<T> {
  const res = await fetch(path, { headers: { accept: "application/json" } });
  const body: unknown = await res.json().catch(() => null);
  if (!res.ok) {
    const message =
      body && typeof body === "object" && "message" in body
        ? String((body as { message: unknown }).message)
        : `Request failed (${res.status})`;
    error(res.status, message);
  }
  return (body as { data: T }).data;
}
