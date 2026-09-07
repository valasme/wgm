import { getCurrentWebview, type Webview } from "@tauri-apps/api/webview";
import { getCurrentWindow, type Window } from "@tauri-apps/api/window";

/**
 * The current Tauri window, or `null` outside Tauri.
 *
 * Every test runs in jsdom with no Tauri runtime behind it, and so does `vite dev` in a
 * plain browser. Returning `null` rather than throwing means the chrome renders and is
 * testable in both, and the window operations are simply no-ops.
 */
export function getWindow(): Window | null {
  if (!inTauri()) {
    return null;
  }

  try {
    return getCurrentWindow();
  } catch {
    return null;
  }
}

/**
 * The current webview, or `null` outside Tauri.
 *
 * Distinct from the window: zoom is a property of the webview, not of the frame around
 * it, and `Window` has no `setZoom`.
 */
export function getWebview(): Webview | null {
  if (!inTauri()) {
    return null;
  }

  try {
    return getCurrentWebview();
  } catch {
    return null;
  }
}

/** Are we running inside the Tauri shell, rather than a bare browser or jsdom? */
export function inTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}
