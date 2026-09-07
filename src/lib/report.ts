import { error as logError, warn as logWarn } from "@tauri-apps/plugin-log";

import { isAppError } from "@/ipc";

/**
 * Getting a failure into the log stream.
 *
 * **Every boundary logs explicitly.** A React error boundary catches its error, so it
 * never reaches `window.onerror` — a design that relies on the global handler alone
 * reports nothing for exactly the errors it was built to catch.
 *
 * Everything here funnels into the same Rust log file, so a Diagnostics Bundle has one
 * stream to read rather than two.
 */

function describe(error: unknown): string {
  if (isAppError(error)) {
    return `${error.code} [${error.correlationId}] ${JSON.stringify(error.context)}`;
  }

  if (error instanceof Error) {
    return `${error.name}: ${error.message}\n${error.stack ?? "(no stack)"}`;
  }

  return String(error);
}

/** Record an error against the boundary or handler that caught it. */
export function reportError(source: string, error: unknown): void {
  void logError(`${source}  ${describe(error)}`);
}

export function reportWarning(source: string, message: string): void {
  void logWarn(`${source}  ${message}`);
}

/**
 * Pipe the console into the Rust log stream, and register the global handlers.
 *
 * `console.warn` and `console.error` are forwarded; `console.log` and `console.debug`
 * are not, in release — they are development noise and would drown the file.
 *
 * React 19's three root handlers are wired in `main.tsx` rather than here, because they
 * are options on `createRoot` rather than global listeners. All three matter:
 * `onRecoverableError` catches concurrent-rendering faults that no boundary ever sees.
 */
export function installGlobalReporting(): void {
  const originalWarn = console.warn;
  const originalError = console.error;

  console.warn = (...args: unknown[]) => {
    originalWarn(...args);
    reportWarning("console", args.map(String).join(" "));
  };

  console.error = (...args: unknown[]) => {
    originalError(...args);
    reportError("console", args.map(String).join(" "));
  };

  window.addEventListener("error", (event) => {
    reportError("window.onerror", event.error ?? event.message);
  });

  window.addEventListener("unhandledrejection", (event) => {
    reportError("unhandledrejection", event.reason);
  });

  // boot.js records anything that failed before the bundle finished loading. Drain it
  // now that a real logger exists — otherwise the one failure that matters most, the
  // one that happened before React, is the one that is never reported.
  const bootErrors = (window as { __wgmBootErrors?: unknown[] }).__wgmBootErrors;
  if (Array.isArray(bootErrors)) {
    for (const entry of bootErrors) {
      reportError("boot", entry);
    }
    bootErrors.length = 0;
  }
}
