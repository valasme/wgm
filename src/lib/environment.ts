import type { ClientEnvironment } from "@/ipc";
import { isHighContrast } from "@/theme/apply-theme";

/**
 * Facts only the webview knows, collected for a Diagnostics Bundle.
 *
 * Rendering and layout bugs are almost always specific to the WebView2 version or the
 * display scaling, so neither is padding. The timezone is here because log timestamps
 * carry a local offset, and a maintainer comparing a user's log against their own
 * otherwise loses an hour to arithmetic.
 *
 * **No environment variables.** Not filtered, not allow-listed — not collected.
 */
export function collectClientEnvironment(): ClientEnvironment {
  return {
    webviewVersion: webviewVersion(),
    displayScaling: `${Math.round((window.devicePixelRatio || 1) * 100)}%`,
    locale: navigator.language,
    timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
    colorScheme: document.documentElement.dataset.theme ?? "unknown",
    highContrast: isHighContrast(),
    windowSize: `${window.innerWidth}×${window.innerHeight}`,
  };
}

/** WebView2 reports itself in the user agent as `Edg/<version>`. */
function webviewVersion(): string {
  return navigator.userAgent.match(/Edg\/([\d.]+)/)?.[1] ?? "unknown";
}
