/**
 * The IPC surface.
 *
 * `bindings.ts` is **generated** by tauri-specta (from `cargo test`) and committed. It
 * is Biome-ignored and must never be hand-edited: a diff you did not intend means a
 * Rust type changed shape, which is the friction ADR-0001 is buying.
 *
 * Everything else imports from here rather than from the generated file, so the
 * generated module has exactly one consumer and the ergonomics live in one place.
 */

import type { AppError, Result } from "./bindings";
import { commands } from "./bindings";

export type * from "./bindings";
export { commands };

/**
 * Unwrap a command result, throwing the `AppError` on failure.
 *
 * The generated bindings return a tagged `Result` so a caller *can* handle failure
 * without a try/catch. Most callers want the exception, because the error boundaries
 * and the toast layer are already built around throwing.
 */
export async function unwrap<T>(promise: Promise<Result<T, AppError>>): Promise<T> {
  const result = await promise;

  if (result.status === "error") {
    throw result.error;
  }

  return result.data;
}

/** Is this an error that came from Rust, rather than a JavaScript one? */
export function isAppError(value: unknown): value is AppError {
  return (
    typeof value === "object" &&
    value !== null &&
    "code" in value &&
    "correlationId" in value &&
    "recoverable" in value
  );
}
