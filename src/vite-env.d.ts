/// <reference types="vite/client" />

/**
 * Globals that `public/boot.js` sets before the bundle runs.
 *
 * Declared rather than guessed at, so the handover between the pre-React script and the
 * app is a typed contract instead of two files that happen to agree.
 */
interface Window {
  /** Failures captured before React existed. Drained by `installGlobalReporting`. */
  __wgmBootErrors?: { kind: string; message: string; source: string }[];
  /** Cleared on first paint. Shows the window regardless after three seconds. */
  __wgmBootFailsafe?: ReturnType<typeof setTimeout>;
}
