import type { Accent, ColorSchemePreference, Density, ReduceMotion } from "@/ipc";
import type { ResolvedColorScheme } from "./accents";

/**
 * Writing Appearance onto `<html>`.
 *
 * Everything visual hangs off four attributes — `data-theme`, `data-accent`,
 * `data-density` and `data-reduce-motion` — and `src/theme/tokens.css` does the rest.
 * Nothing here touches a colour: that is the whole point of ADR-0002's attribute
 * contract, and it is what lets the boot script below run before React exists.
 */

export interface Appearance {
  colorScheme: ColorSchemePreference;
  accent: Accent;
  density: Density;
  reduceMotion: ReduceMotion;
}

const MEDIA_DARK = "(prefers-color-scheme: dark)";

/**
 * Resolve the preference to a scheme that can actually be in effect.
 *
 * `System` resolves to Light or Dark and **never** to Darker: the operating system has
 * no signal for "extra dark", so inferring one would mean guessing.
 */
export function resolveColorScheme(preference: ColorSchemePreference): ResolvedColorScheme {
  if (preference !== "system") {
    return preference;
  }

  return window.matchMedia?.(MEDIA_DARK).matches ? "dark" : "light";
}

/** Apply Appearance to the document element. Idempotent, and safe to call on every change. */
export function applyAppearance(
  appearance: Appearance,
  root: HTMLElement = document.documentElement,
): void {
  root.dataset.theme = resolveColorScheme(appearance.colorScheme);
  root.dataset.accent = appearance.accent;
  root.dataset.density = appearance.density;
  root.dataset.reduceMotion = appearance.reduceMotion;
}

/**
 * Re-resolve when Windows switches between light and dark.
 *
 * Only matters while the preference is `system`; the caller re-subscribes when the
 * preference changes. Returns an unsubscribe function.
 */
export function watchSystemColorScheme(
  onChange: (scheme: ResolvedColorScheme) => void,
): () => void {
  const query = window.matchMedia?.(MEDIA_DARK);

  if (!query) {
    return () => {};
  }

  const listener = (event: MediaQueryListEvent) => onChange(event.matches ? "dark" : "light");

  query.addEventListener("change", listener);
  return () => query.removeEventListener("change", listener);
}

/** Is Windows High Contrast on? Reported in a Diagnostics Bundle, never designed around. */
export function isHighContrast(): boolean {
  return window.matchMedia?.("(forced-colors: active)").matches ?? false;
}

/**
 * Where the zero-flash boot script reads Appearance from.
 *
 * Rust reads Settings before the window is created and sets the native background
 * colour (ADR-0001), but the *document* still paints with whatever `tokens.css`
 * defaults to until React mounts. On a dark install that is one white frame on every
 * launch, which looks broken.
 *
 * So `public/boot.js` writes the attributes synchronously, from the value the previous
 * run left here. This is the one piece of state the webview keeps of its own, and it
 * is a **cache, not a source of truth**: Rust still owns Settings, and React overwrites
 * whatever this guessed as soon as it has the real answer.
 *
 * The script is a file rather than an inline `<script>` because the
 * Content-Security-Policy sets `script-src 'self'` and an inline one would be blocked.
 */
export const BOOT_STORAGE_KEY = "wgm.appearance";

export function rememberAppearance(appearance: Appearance): void {
  try {
    localStorage.setItem(BOOT_STORAGE_KEY, JSON.stringify(appearance));
  } catch {
    // A private window, cleared site data, or storage disabled entirely. The cost is
    // one flash on the next launch, so there is nothing to report and nothing to do.
  }
}
