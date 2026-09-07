import type { AppError, Result } from "@/ipc";

/**
 * Turn a command's tagged `Result` into a promise that throws.
 *
 * The settings store's rollback path is built around a rejected promise, and the
 * generated bindings hand back a discriminated union. This is the one adapter between
 * the two, rather than the same four lines in every settings page.
 */
export async function commit<T>(promise: Promise<Result<T, AppError>>): Promise<T> {
  const result = await promise;

  if (result.status === "error") {
    throw result.error;
  }

  return result.data;
}
