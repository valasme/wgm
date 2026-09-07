/*
 * Runs before the bundle, synchronously, with `script-src 'self'` satisfied.
 *
 * Three jobs, and every one of them only matters when something else has gone wrong:
 *
 * 1. **Zero-flash boot.** Write the Appearance attributes from the value the previous
 *    run cached, so the first painted frame is already the right colour scheme. React
 *    overwrites all of this the moment it has the real Settings from Rust — the cache
 *    is a guess, not a source of truth.
 *
 * 2. **Catch a failure that happens before React exists.** A React error boundary
 *    cannot catch an error thrown while the bundle is still loading, because React
 *    never started. These handlers are registered first, so a boot failure reveals the
 *    static fallback in index.html instead of leaving a blank white window — the worst
 *    failure in the app.
 *
 * 3. **The boot failsafe.** The window is created hidden and shown on first paint; if
 *    first paint never happens, this shows it anyway.
 *
 * No imports and no bundler: this file has to run even when the bundle is the thing
 * that is broken. It targets WebView2, which is evergreen Chromium, so modern syntax
 * is fine — the constraint is the absence of a build step, not the language level.
 */
(() => {
  const STORAGE_KEY = "wgm.appearance";

  // --- 1. Appearance ------------------------------------------------------
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    const saved = raw ? JSON.parse(raw) : null;
    const root = document.documentElement;

    let scheme = saved?.colorScheme ?? "system";
    if (scheme === "system") {
      // `system` resolves to light or dark and never to darker: the OS has no signal
      // for "extra dark", so inferring one would mean guessing.
      scheme = window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
    }

    root.dataset.theme = scheme;
    root.dataset.accent = saved?.accent ?? "blue";
    root.dataset.density = saved?.density ?? "comfortable";
    root.dataset.reduceMotion = saved?.reduceMotion ?? "system";
  } catch {
    // Storage unavailable, or a cached value from a build that shaped it differently.
    document.documentElement.dataset.theme = "light";
    document.documentElement.dataset.accent = "blue";
  }

  // --- 2. Pre-React failure capture ---------------------------------------
  window.__wgmBootErrors = [];

  const record = (kind, message, source) => {
    window.__wgmBootErrors.push({ kind, message: String(message), source: String(source) });

    const fallback = document.getElementById("boot-fallback");
    if (fallback) {
      fallback.hidden = false;
      const detail = document.getElementById("boot-fallback-detail");
      if (detail) {
        detail.textContent = String(message);
      }
    }
  };

  window.addEventListener("error", (event) => {
    record("error", event.message || event.type, event.filename || "");
  });

  window.addEventListener("unhandledrejection", (event) => {
    record("unhandledrejection", event.reason?.message ?? event.reason, "");
  });

  // --- 3. The failsafe ----------------------------------------------------
  // Both halves are required. The failsafe without the fallback markup above reveals a
  // blank white window; the fallback without the failsafe is never seen at all.
  window.__wgmBootFailsafe = setTimeout(() => {
    window.__TAURI_INTERNALS__?.invoke("show_window", {}).catch(() => {});
  }, 3000);
})();
