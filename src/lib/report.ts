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

/** Guards against a *synchronous* re-entry — the log call itself reaching `console.error`. */
let reporting = false;

/**
 * Send one line to the Rust log stream.
 *
 * **Logging must never become the thing that breaks the app**, so this swallows every
 * way it can fail, and each guard covers a different one:
 *
 * - The `.catch` handles the *asynchronous* failure. Without it a rejected `logError`
 *   surfaces as an `unhandledrejection`, whose handler reports it, which rejects
 *   again — an unbounded loop that runs on every call whenever the log command is
 *   unavailable, which is every call outside Tauri.
 * - The `reporting` flag handles the *synchronous* one: if the log path ever reaches
 *   `console.error`, the override below would call straight back into here.
 */
function send(write: (message: string) => Promise<void>, line: string): void {
  if (reporting) {
    return;
  }

  reporting = true;
  try {
    void write(line).catch(() => {});
  } finally {
    reporting = false;
  }
}

/** Record an error against the boundary or handler that caught it. */
export function reportError(source: string, error: unknown): void {
  send(logError, `${source}  ${describe(error)}`);
}

export function reportWarning(source: string, message: string): void {
  send(logWarn, `${source}  ${message}`);
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
