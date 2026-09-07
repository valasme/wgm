# wgm — Failure Modes and Fallbacks

Every way wgm can fail, and what it does instead of breaking. A scaffold with no features
still has a large failure surface, because all of it is I/O, process boot and window
management.

Principle throughout: **never destroy the user's data, never die silently, and never leave a
window with nothing in it.**

---

## 1. Boot

Boot is the only phase where a failure leaves the user with no interface to explain it, so it
gets the most fallbacks.

| Failure | Behaviour |
| --- | --- |
| JS bundle fails to load or throws before React mounts | `index.html` ships **static fallback markup and a `window.onerror` handler registered before the bundle**. React replaces it on mount. Without this, the boot failsafe below reveals a permanently blank white window — the worst failure in the app, and the one a React error boundary cannot catch because React never started. |
| React mounts but throws during first render | Root error boundary → crash screen |
| Window never reaches first paint | **3-second failsafe shows the window regardless.** Combined with the row above, the user sees a real message rather than nothing |
| Rust panics before the window exists | Panic hook writes to the log, then the process aborts. Nothing else is possible; the log is the only artefact |
| Neither `./data` nor `%APPDATA%` is writable | **The app still starts**, in ephemeral mode: Settings live in memory, a persistent banner says changes will not be saved and why. Refusing to launch because a directory is read-only is a worse outcome than launching degraded |
| Portable marker present but the folder is read-only (e.g. under `Program Files`) | Fall back to `%APPDATA%`, log a warning, show the resolved path in About |
| WebView2 missing | Tauri's own error. Outside our control; the README documents the requirement |

## 2. The settings file

| Condition | Behaviour |
| --- | --- |
| Unparseable | Copy to `settings.corrupt-<ts>.json`, boot from Defaults, persistent banner naming the saved file with an Open-folder button. **Never delete it** |
| Missing fields | Fill from Defaults, log `warn`, rewrite |
| Unknown fields | Ignore, log — protects users who downgrade |
| Out-of-range value | Clamp to range, log; do not reject the whole file |
| `schemaVersion` newer than the app | Do not guess. Boot from Defaults, explain the downgrade, do not overwrite the file |
| Older `schemaVersion` | Run the migration chain, then rewrite |
| **A migration itself fails or panics** | Treat exactly as corrupt: back up, boot from Defaults, banner. Migrations are pure and total, and each has a checked-in fixture |
| **File locked on write** (antivirus scan, OneDrive sync, backup agent) | Retry with backoff. This is common on Windows, not exotic |
| **Repeated write failures** | Coalesce into **one persistent banner**, not one toast per keystroke. Instant apply means every toggle writes, and a locked file would otherwise produce a wall of toasts |
| Disk full | Distinct error code and message; the file on disk is untouched because writes are atomic |
| Externally edited while running | wgm will overwrite on next write. **Accepted limitation**, documented rather than watched — file watching to solve it costs more than it returns |

**All document writes pass through a single lock.** Reset-to-defaults, an import, and a
debounced sidebar-width write can otherwise interleave and produce a file that is neither.

## 3. Import and export

| Failure | Behaviour |
| --- | --- |
| Invalid or hostile import file | Validate every field, **reject atomically**. Nothing is ever partially applied |
| Backup cannot be written | **Abort the import.** `settings.backup.json` is the undo; without it the import is not safe to perform |
| Import from a newer schema | Refuse with an explanation rather than guessing |
| Import from an older schema | Migrate, then show a preview — *"this will change 4 settings"* — before applying |
| Export target not writable | Error naming the path; the picker reopens |

Workspace State is structurally excluded from both, so window geometry, Sidebar width and
onboarding progress cannot leak into an export or be imported from someone else's machine.

## 4. Logging and diagnostics

| Failure | Behaviour |
| --- | --- |
| Log file cannot be written | Warn once, then fail silently. Logging must never become the thing that breaks the app |
| Retention deletion fails | Log and continue |
| Retention cannot keep up | A **size cap as well as a day cap** — a Trace-level session can outgrow a 7-day window in hours |
| Bundle creation fails (disk full, zip error) | Its own error path and message; no partial `.zip` is left behind |

**Redaction is deny-by-default and does not rely on matching `C:\Users\`.** That pattern
misses redirected profiles, `D:\Users`, UNC home directories and domain-joined machines
entirely. Instead: resolve the real profile directory from the OS, and redact that path, the
bare username wherever it appears, and the `USERNAME`/`USERPROFILE` environment values. If
the username cannot be determined, say so in the bundle rather than shipping an unredacted
one.

The crash screen's **Copy diagnostics** runs the same redaction. It is the more likely of the
two to be pasted into a public issue.

## 5. The wgm Release check

The only network call in the product, so its failures are all user-visible and all need copy.

| Condition | Message |
| --- | --- |
| A newer release exists | Version number and a link to the release page |
| Already current | You're on the latest version |
| Offline / DNS failure | Couldn't reach GitHub. Check your connection and try again |
| GitHub returns 5xx | GitHub isn't responding right now. Try again later |
| **Rate limited (403)** | Unauthenticated GitHub API allows 60 requests an hour per IP. Disable the button until the reset time and say when it will work again |
| Timeout | 10-second cap, then treated as offline |

## 6. IPC and unexpected errors

Every command returns `Result<T, AppError>` carrying a stable `code`, structured context,
`recoverable`, and a `correlation_id`.

**The IPC wrapper normalises anything that is not an `AppError` into one**, with code
`IPC_UNKNOWN` — a Tauri transport failure, a serialisation error, or a panic crossing the
boundary does not arrive in our shape, and a raw string reaching the UI means no translation,
no correlation id, and no recovery action.

**The correlation id is displayed**, not just logged — small, monospace, selectable, on the
crash screen and in every non-trivial error. A user quoting `a3f9c1` is the only way to find
their failure in a bundle; an id that exists solely in a log file the user never opens is not
a diagnostic.

## 7. Error boundaries

| Layer | Catches | Recovery offered |
| --- | --- | --- |
| Root | Anything React throws below it | Copy diagnostics · Open log folder · Restart. **Inline styles only** — a failure in the theme layer would otherwise take the crash screen down with it |
| Route | A route's render or loader | **Reset the boundary** (TanStack's `errorComponent` `reset`), not just navigation. Title Bar and Sidebar stay alive so the user can leave |
| Section | One settings panel | Retry that panel |
| Global capture | `window.onerror`, `unhandledrejection`, Rust panic hook | Funnelled into one log stream with shared correlation ids |

A dev-only `debug_panic` command exists behind `#[cfg(debug_assertions)]` so the panic path
is testable rather than theoretical.

## 8. Onboarding

If Settings cannot be written during onboarding, completion never persists and setup replays
on every launch — a small failure that produces a maddening symptom. Detect the failed write,
finish onboarding anyway, and show the ephemeral-mode banner from §1.
