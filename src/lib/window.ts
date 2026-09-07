import { getCurrentWindow, type Window } from "@tauri-apps/api/window";

/**
 * The current Tauri window, or `null` outside Tauri.
 *
 * Every test runs in jsdom with no Tauri runtime behind it, and so does `vite dev` in a
 * plain browser. Returning `null` rather than throwing means the chrome renders and is
 * testable in both, and the window operations are simply no-ops.
 */
export function getWindow(): Window | null {
  if (typeof window === "undefined" || !("__TAURI_INTERNALS__" in window)) {
    return null;
  }

  try {
    return getCurrentWindow();
  } catch {
    return null;
  }
}

/** Are we running inside the Tauri shell, rather than a bare browser or jsdom? */
export function inTauri(): boolean {
  return getWindow() !== null;
}
