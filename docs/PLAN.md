# wgm — Scaffold Implementation Plan

**Revision 6.** The goal is a **complete, production-shaped foundation with no product
features**. Settings is the only fully-implemented feature; every other route renders its
real empty state over no data.

Read `CONTEXT.md` for vocabulary, `docs/design.md` for the palette, type scale, layout rules
and every user-facing string, `docs/accessibility.md` for the a11y contract,
`docs/failure-modes.md` for what happens when things break, `docs/error-reporting.md` for how
a failure reaches a maintainer, and `docs/adr/` for the three decisions that constrain
everything below.

§6 is the revision log. Where the body and the log disagree, the body wins — the log records
history, not requirements.

---

## 0. Decisions already settled

| Area | Decision |
| --- | --- |
| Stack | Tauri 2 · React 19.1 · TypeScript 6 · Vite 8 · pnpm |
| Styling | Tailwind v4 (`@theme` + OKLCH CSS variables) |
| Components | shadcn/ui — copied into the repo, built on Radix |
| Routing | TanStack Router, file-based, **hash history**, `autoCodeSplitting` |
| Client state | Zustand. **No TanStack Query** until real remote data exists |
| IPC | `tauri-specta` → generated `src/ipc/bindings.ts` |
| Persisted state | Rust-owned Versioned Documents (ADR-0001) |
| Color Schemes | Light · Dark · Darker, `data-theme` attribute (ADR-0002) |
| Accents | 8 fixed, OKLCH, AA-verified in CI |
| Network | Zero, except a manual wgm Release check **made from Rust** (ADR-0003) |
| Lint/format | Biome. React Compiler on, with a healthcheck in CI |
| Tests | Vitest + Testing Library + axe-core. No E2E yet |
| License | MIT, with the name and logo carved out |
| Signing | Unsigned; CI has a no-op signing step and publishes SHA256 checksums |
| Min OS | Windows 10 1809 (build 17763), matching winget's own floor |

## 1. Known defects in the current template — fix in Phase 0

1. `tauri.conf.json` sets `"csp": null`, disabling the Content-Security-Policy entirely.
2. `index.html` has the template title and links a favicon at `/vite.svg`, absent from `public/`.
3. `Cargo.toml` has `authors = ["you"]` and `description = "A Tauri App"`.
4. `bundle.targets: "all"` is unspecific.
5. `panic = "abort"` kills the whole process when a single command panics.
6. **`strip = true` with no `debug` setting** makes every future panic backtrace a column of
   hex addresses.
7. No `LICENSE` file, which legally means all-rights-reserved on a repo intended to be open source.
8. `tsconfig` targets ES2020 despite shipping to exactly one evergreen engine (WebView2).

---

## 2. Dependencies

### Frontend — runtime

```
@tanstack/react-router
zustand
lucide-react
sonner
cmdk
class-variance-authority  clsx  tailwind-merge
@tauri-apps/api
@tauri-apps/plugin-autostart          # "launch on startup"
@tauri-apps/plugin-clipboard-manager  # Copy summary, Copy diagnostics
@tauri-apps/plugin-dialog             # native file picker for import/export
@tauri-apps/plugin-log
@tauri-apps/plugin-opener             # "Open log folder", external links
@tauri-apps/plugin-os
@tauri-apps/plugin-process            # relaunch from the crash screen
@fontsource-variable/geist
```

Radix packages arrive individually as `shadcn add` pulls them in.

**Deliberately absent:**

- **Zod** — Rust validates at the trust boundary; a client-side schema would be duplicated
  logic that can silently disagree with the Rust one.
- **`@tauri-apps/plugin-window-state`** — it saves to the app config dir with no path
  override, splitting state across two directories in portable mode. Window geometry is a
  Workspace State field in our own document instead.
- **`@fontsource-variable/geist-mono`** — no monospace glyph is rendered anywhere in the
  scaffold. Added when Package identifiers first appear on screen, not before.

### Frontend — dev

```
vite  @vitejs/plugin-react  @tanstack/router-plugin  @tailwindcss/vite
tailwindcss  typescript  tsx  @biomejs/biome  babel-plugin-react-compiler
react-compiler-healthcheck
vitest  @vitest/coverage-v8  jsdom
@testing-library/react  @testing-library/user-event  @testing-library/jest-dom
axe-core  vitest-axe
size-limit  @size-limit/preset-app
lefthook
@tauri-apps/cli
```

`tsx` runs `scripts/check-contrast.ts` — TypeScript alone cannot execute a `.ts` file.

### Rust

```toml
tauri = { version = "2" }
tauri-plugin-autostart         = "2"
tauri-plugin-clipboard-manager = "2"
tauri-plugin-opener  = "2"
tauri-plugin-dialog  = "2"
tauri-plugin-log     = "2"
tauri-plugin-os      = "2"
tauri-plugin-process = "2"
tauri-plugin-http    = "2"   # the wgm Release check, Rust-side only
tauri-plugin-single-instance = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
specta = "2"
specta-typescript = "0"
tauri-specta = { version = "2", features = ["derive", "typescript"] }
thiserror = "2"
log = "0.4"
uuid = { version = "1", features = ["v4"] }   # Correlation Ids, Session Ids
time = "0.3"
zip = "2"
tempfile = "3"

[features]
default = []
devtools = ["tauri/devtools"]   # dev builds only — never in a release artifact

[profile.release]
codegen-units = 1
lto = true
opt-level = 3
panic = "unwind"    # a panicking command must not kill the whole app
debug = 1           # line tables → a .pdb → symbolisable backtraces
strip = "debuginfo" # keep the symbol table in the exe; the .pdb ships separately

[target.'cfg(windows)'.dependencies]
windows = { version = "0", features = [
  "Win32_Foundation", "Win32_Graphics_Gdi",
  "Win32_UI_WindowsAndMessaging", "Win32_UI_Shell",
] }
```

`tauri-plugin-updater` is **intentionally not listed** — see ADR-0003.

Also required: **`rust-toolchain.toml`** pinning the exact toolchain, or `clippy -D warnings`
breaks CI on an unrelated day when the runner picks up a new Rust release. And a
`packageManager` field in `package.json` with corepack, so contributors don't churn the
lockfile across pnpm versions.

Tooling installed separately: `cargo-about`, `release-please` (GitHub Action).

---

## 3. Target file tree

```
wgm/
├── .github/
│   ├── workflows/{ci.yml, release.yml, licenses.yml}
│   ├── ISSUE_TEMPLATE/{bug_report.yml, feature_request.yml, config.yml}
│   ├── PULL_REQUEST_TEMPLATE.md
│   └── dependabot.yml            # npm + cargo + github-actions
├── release-please-config.json  .release-please-manifest.json
├── docs/
│   ├── adr/0001–0003             # written
│   ├── agents/                   # written
│   ├── PLAN.md                   # this file
│   ├── design.md  accessibility.md  failure-modes.md  error-reporting.md
│   └── development.md            # written in Phase 1; holds the manual test passes
├── licenses/                     # OFL text for Geist and Instrument Serif
├── public/favicon.png
├── index.html                    # carries the pre-React static failure fallback
├── scripts/
│   ├── check-contrast.ts         # accents AND the neutral ramp, 3 schemes
│   └── subset-wordmark.md        # how to regenerate the Instrument Serif subset
├── src/
│   ├── main.tsx                  # React 19 root: onUncaughtError/onCaughtError/onRecoverableError
│   ├── routeTree.gen.ts          # GENERATED, committed, Biome-ignored
│   ├── routes/
│   │   ├── __root.tsx
│   │   ├── index.tsx                     → redirect to /search
│   │   ├── onboarding.tsx
│   │   ├── _app.tsx                      Title Bar + Sidebar + Content Area
│   │   ├── _app/search.tsx  kits.index.tsx  kits.$kitId.tsx
│   │   ├── _app/installed.tsx  updates.tsx          # Package Updates
│   │   ├── _app/settings.tsx             nested layout, owns the <h1>
│   │   ├── _app/settings.index.tsx       → redirect to /settings/appearance
│   │   └── _app/settings.{appearance,general,advanced,about}.tsx
│   ├── components/
│   │   ├── ui/                   # shadcn output
│   │   ├── chrome/{title-bar,window-controls,sidebar,sidebar-resizer,wordmark}.tsx
│   │   ├── command-palette.tsx
│   │   ├── states/{page-placeholder,empty-state,loading-state,error-state}.tsx
│   │   ├── errors/{root-boundary,route-error,section-error,crash-screen}.tsx
│   │   ├── onboarding/{step-welcome,step-appearance,step-privacy,step-ready}.tsx
│   │   └── settings/{setting-row,setting-section,setting-switch,setting-select,
│   │                 recent-problems,bundle-preview}.tsx
│   ├── ipc/{bindings.ts (GENERATED, Biome-ignored), index.ts}
│   ├── stores/{settings-store,workspace-store}.ts
│   ├── theme/{tokens.css, accents.ts, apply-theme.ts}
│   ├── i18n/{en.ts, t.ts}
│   ├── lib/{shortcuts,focus,report,cn}.ts
│   ├── styles/global.css
│   └── test/{setup.ts, a11y.test.tsx}
├── src-tauri/
│   ├── src/
│   │   ├── main.rs   lib.rs
│   │   ├── error.rs                      AppError, codes, Correlation Ids
│   │   ├── logging.rs                    format, session banner, flush policy
│   │   ├── ring.rs                       in-memory Trace buffer
│   │   ├── paths.rs                      %APPDATA% vs portable, writability probe
│   │   ├── document/{mod.rs, atomic.rs}  the generic Versioned Document module
│   │   ├── settings/{mod,schema,defaults,migrations,commands}.rs
│   │   ├── workspace/{mod,schema,defaults,migrations,commands}.rs
│   │   ├── diagnostics/{mod,bundle,manifest,summary,redact}.rs
│   │   ├── releases.rs                   manual wgm Release check, Rust-side HTTP
│   │   └── platform/windows/{mod,window_proc,system_menu}.rs
│   ├── tests/fixtures/{settings,workspace,redaction}/*
│   ├── capabilities/default.json
│   └── {Cargo.toml, tauri.conf.json}
├── AGENTS.md  CLAUDE.md  CONTEXT.md  README.md
├── LICENSE  THIRD-PARTY-LICENSES.md  SECURITY.md
├── CONTRIBUTING.md  CODE_OF_CONDUCT.md
├── biome.json  lefthook.yml  components.json  rust-toolchain.toml  .editorconfig
└── .size-limit.json  vitest.config.ts  vite.config.ts  package.json
```

---

## 4. Phases

From Phase 1 onward each phase ends green:
`pnpm check && pnpm typecheck && pnpm test && cargo clippy -- -D warnings`.
Phase 0 predates those scripts and has its own gate.

### Phase 0 — Hygiene and legal

Fix all eight defects in §1, including the release profile in §2.

Add `LICENSE` (MIT, Dimitris Valasellis, 2026) with an explicit carve-out — *the MIT grant
covers the source code; the wgm name, wordmark and icon are not licensed for use in derivative
works* — plus `SECURITY.md` (private reporting via GitHub Security Advisories; state plainly
that there is no auto-update, so users must watch releases), `CONTRIBUTING.md`, and
`CODE_OF_CONDUCT.md` (Contributor Covenant 2.1, with a real contact address).

Rewrite `README.md`: what wgm is, the Windows 10 1809 + WebView2 requirement, install
instructions, an honest explanation of the SmartScreen warning and how to verify
`SHA256SUMS.txt`, the zero-telemetry promise, and the Microsoft disclaimer.

CSP:

```
default-src 'self'; script-src 'self'; connect-src 'self' ipc: http://ipc.localhost;
img-src 'self' data:; font-src 'self'; style-src 'self' 'unsafe-inline';
object-src 'none'; base-uri 'none'; frame-src 'none'
```

`style-src 'unsafe-inline'` is unavoidable — Radix positions floating elements with inline
style attributes. `connect-src` deliberately excludes GitHub: the wgm Release check happens
in Rust, not the webview.

Window: `1200×760`, minimum `680×480`, `decorations: false`, `visible: false`, `shadow: true`,
`bundle.targets: ["nsis"]`.

**Done when:** `git grep -i "A Tauri App\|authors = \[\"you\"\]" -- ':!docs'` returns nothing
and `LICENSE` exists. The path exclusion matters: this plan document quotes the strings it is
asking you to delete.

### Phase 1 — Tooling

Biome, **ignoring `src/routeTree.gen.ts` and `src/ipc/bindings.ts`** so formatting doesn't
fight the code generators. Lefthook running Biome + `cargo fmt --check` + `clippy -D warnings`
on staged files. Vitest with jsdom, Testing Library and `vitest-axe`. `rust-toolchain.toml`,
`.editorconfig`, the `packageManager` field, and `docs/development.md` holding the manual
test passes that CI cannot run.

React Compiler through `@vitejs/plugin-react`'s babel option, **plus `react-compiler-healthcheck`
in CI** — Biome has no `react-compiler` rule, so a component that silently bails out of
compilation is otherwise invisible.

Vite plugin order matters: `tanstackRouter({ target: 'react', autoCodeSplitting: true })`
must be registered **before** `@vitejs/plugin-react`. **Source maps are enabled for release
builds** — an open-source project has nothing to protect by withholding them, and a minified
stack trace is not a bug report.

`ci.yml` splits by cost: typecheck, Biome, Vitest, `check-contrast` and the compiler
healthcheck on `ubuntu-latest`; `cargo fmt`, `clippy`, `cargo test` and `tauri build` on
`windows-latest` with `Swatinem/rust-cache`. `size-limit` is wired but **unenforced** — it
gets a real number in Phase 4.

**Done when:** a deliberately-broken commit is blocked by the pre-commit hook and by CI.

### Phase 2 — Rust foundation

The load-bearing phase. Nothing visual.

- **`paths.rs`** — `./data/` when a `wgm.portable` file sits beside the executable, otherwise
  `%APPDATA%\io.github.valasme.wgm\`. **Probe writability first**: a portable install under
  `Program Files` cannot write beside itself, so fall back to `%APPDATA%` and log a warning.
  If *neither* is writable, enter **Ephemeral Mode** (`docs/failure-modes.md` §1) rather than
  refusing to launch. The resolved path and mode appear in Settings → About. The log plugin
  uses `TargetKind::Folder { path }` pointing at the *same* resolved directory.
- **`error.rs`** — `AppError` via `thiserror`, carrying a stable screaming-snake `code`,
  structured `context`, `recoverable: bool` and a `correlation_id`. **No English crosses the
  IPC boundary**; `src/i18n/en.ts` owns every word.
- **`logging.rs` + `ring.rs`** — the fixed line format, per-run session banner, daily rotation
  with both a day cap and a size cap, immediate flush on `error` and in the panic hook, and
  the 500-record in-memory Trace ring buffer that is captured regardless of the configured
  file level. Frontend `console.warn`/`console.error` pipe into the same stream. The panic
  hook calls `Backtrace::force_capture()` and must not itself panic. All per
  `docs/error-reporting.md` §1–2.
- **`document/`** — the generic Versioned Document module: `load`, `save`, `import`, `export`,
  `migrate`, with `atomic.rs` doing temp-file + fsync + rename, and **a single write lock**
  shared across every document.
- **`settings/`** and **`workspace/`** — two documents on one module. Settings holds
  everything meaningful on any machine and is exportable; Workspace State holds window
  geometry, Sidebar width, Rail state and onboarding progress, and is never exported. The
  split makes the export exclusion structural rather than a list someone forgets to update,
  and two consumers is what proves the module generic. Settings fields carry `#[redact]`
  where a future value could be a secret.
- **`diagnostics/`** — bundle, manifest, markdown summary, and redaction. Redaction resolves
  the real profile path from the OS rather than matching `C:\Users\`; environment variables
  are never collected; `COMPUTERNAME` is redacted.
- **`releases.rs`** — the wgm Release check. One function, `tauri-plugin-http`, no identifiers
  sent, 10-second timeout, five failure states per `docs/failure-modes.md` §5.
- **`platform/windows/`** — one window subclass handling **both** `WM_NCHITTEST` (return
  `HTMAXBUTTON` over the maximise button so Snap Layouts still appears) **and
  `WM_GETMINMAXINFO`** (clamp maximised size to the monitor *work area*, without which an
  undecorated window covers the taskbar). `system_menu.rs` restores `Alt+Space`. Both behind
  `#[cfg(windows)]` and one `const ENABLE_NATIVE_HIT_TEST: bool` kill-switch.
- **Capabilities** — a security surface, enumerated explicitly: `core:default`,
  `core:window:allow-start-dragging`, `allow-minimize`, `allow-toggle-maximize`,
  `allow-is-maximized`, `allow-close`, `allow-set-theme`, `allow-set-title`,
  `dialog:allow-open`, `dialog:allow-save`, `opener:allow-open-path`, `opener:allow-open-url`,
  `clipboard-manager:allow-write-text`, `os:allow-*`, `process:allow-restart`,
  `autostart:default`. **The `fs` plugin is never enabled.**
- **Dev-only commands** behind `#[cfg(debug_assertions)]`: `debug_panic` and `debug_error`, so
  the whole reporting path is exercisable rather than theoretical.
- `tauri-specta` emits `src/ipc/bindings.ts` on build.

**Document resilience — each row a fixture test, for both documents:**

| Condition | Behaviour |
| --- | --- |
| Unparseable | Copy to `settings.corrupt-<ts>.json`, boot from Defaults, persistent banner naming the file. **Never delete it.** |
| Missing fields | Fill from Defaults, log `warn`, rewrite |
| Unknown fields | Ignore, log — protects users who downgrade |
| Out-of-range value | Clamp to range, log; don't reject the whole file |
| `schemaVersion` newer than the app | Don't guess. Boot from Defaults in Ephemeral Mode; don't overwrite the file |
| Older `schemaVersion` | Run the migration chain, then rewrite |
| A migration fails or panics | Treat exactly as unparseable |
| File locked (antivirus, OneDrive) | Retry with backoff; coalesce repeated failures into one banner, never one toast per toggle |

**Done when:** `cargo test` covers every row for both documents, the redaction golden-file
test passes, and `bindings.ts` regenerates cleanly.

### Phase 3 — Theme, type and copy

`tokens.css` declaring the OKLCH variables for Light, Dark and Darker under `[data-theme]`,
with `color-scheme` set correctly on each. Exact values in `docs/design.md` §2. Darker is
**drawn, not derived** — on true black a raised `--surface` and a larger `--border` step are
required, because a design built on hairlines stops working if you scale Dark's lightness down.

`accents.ts` defines the eight Accents (Blue default · Violet · Cyan · Emerald · Amber ·
Orange · Rose · Neutral). `scripts/check-contrast.ts` fails CI on **both** the accent pairs
**and** the neutral text ramp, across all three schemes, at WCAG AA — the neutral ramp is most
of the pixels and gating only the accents would leave it unverified.

Reduced motion is driven by `[data-reduce-motion]` on `<html>` *as well as* the media query,
so the setting can override the OS in both directions.

Fonts, self-hosted: Geist Sans Variable via Fontsource, and a checked-in Instrument Serif
Italic subset containing only `w`, `g`, `m`. Both `preload`ed with `font-display: swap`.
`licenses/` carries the OFL text for both — the licence requires it, and a subset counts as a
Modified Version.

**`i18n/en.ts` and the typed `t()` land here**, not later: Phase 4's empty states and Phase 5's
settings labels both need it, and retrofitting a message catalog after the strings exist is
the exact work the catalog was meant to avoid. `t()` takes typed interpolation parameters from
day one, because Rust error codes arrive carrying structured context like `{ path }`.

The zero-flash boot script from ADR-0001 is implemented here, since it depends on the
`data-theme` contract above.

**Done when:** the contrast script passes and toggling `data-theme` in devtools restyles the
whole app with no hardcoded colours anywhere.

### Phase 4 — Application shell

- **Title Bar** (36px): `PanelLeft` ⇄ `PanelLeftClose` toggle, Wordmark, Window Controls. The
  controls are three hand-drawn 10×10 SVGs on `currentColor`, matching Segoe Fluent Icons
  metrics — not lucide, whose rounded `Square` reads visibly wrong as a maximise button.
  Maximise swaps to restore on window state. They form one `role="toolbar"` (Radix Toolbar)
  with a single tab stop and arrow-key navigation, **in DOM order matching visual order** —
  repositioning them with CSS would make focus jump across the window, a WCAG 2.4.3 failure.
  `data-tauri-drag-region` on the bar, double-click to maximise, drag-to-unsnap. The OS window
  title updates on every route change.
- **Sidebar**: default 220px, drag-resizable 180–400px with a **10px hit area**, collapsing to
  a 52px Rail. The resizer implements the full ARIA splitter pattern (`role="separator"`,
  `aria-valuenow`/`min`/`max`/`text`); arrow keys move 16px, Home/End jump to the bounds.
  Width persists to Workspace State, live during drag, debounced 300 ms. **Below 900px window
  width the Sidebar forces to Rail.** Rail items carry tooltips *and* `aria-label`s.
- **Router**: hash history, `autoCodeSplitting`, `scrollRestoration`. Nav order — Search ·
  Kits · Installed · Updates, Settings pinned to the bottom.
- **Command Palette**: `Ctrl+K`, triggered by the Sidebar pill (magnifier + "Search" + a
  `Ctrl K` hint). Navigates routes, toggles Color Scheme, jumps to any setting.
- **State primitives**: `<PagePlaceholder>`, `<EmptyState>`, `<LoadingState>`, `<ErrorState>`.
  **Each placeholder route renders its real empty state**, with the copy fixed in
  `docs/design.md` §8 — not "coming soon" text. The scaffold is almost entirely empty states,
  so they are the product today, and they cost the same to write well. Every route has a
  heading for Phase 7's focus management.
- **Boot resilience**, both halves: `index.html` ships **static fallback markup and a
  `window.onerror` handler registered before the bundle**, and the window shows on first paint
  with a **3-second failsafe**. Either alone is insufficient — the failsafe without the
  fallback reveals a blank white window, which is the worst failure in the app.

Now measure the real bundle and set `.size-limit.json` to that number plus 10% headroom,
enforced from here on.

**Done when:** every route renders, the Rail collapses, the resizer works by keyboard, Snap
Layouts appears on maximise-button hover, and maximising does not cover the taskbar.

### Phase 5 — Settings

**Four pages**, **instant apply**: the control moves immediately, Rust confirms, and on failure
it rolls back — announced three ways per `docs/accessibility.md` §8, since a silent revert
lies to a screen reader that has already read the new state. No Save button, no dirty state,
no navigation guards.

- **Appearance** — Color Scheme (System/Light/Dark/Darker) · Accent · density · reduce motion
- **General** — restore window position · confirm before destructive actions
  (*launch on startup* and *start minimised* were dropped: wgm writes no registry run entry
  and never opens into the taskbar, and `plugin-autostart` went with them)
- **Advanced** — log level · log retention · Open log folder · Export diagnostics ·
  Recent problems · Import settings · Export settings · Reset to defaults
- **About** — version, commit SHA, build date, Tauri and WebView2 versions, data directory
  path and mode + Open folder, licences, *Check for a new version*, *Show setup again*

Every user-facing string is fixed in `docs/design.md` §8. Sidebar width is not a settings row —
it lives in Workspace State and is adjusted by dragging. Two controls follow the
no-dead-controls rule: **language is not rendered** until a second catalog exists, and
**confirm-before-destructive-actions is rendered only because it genuinely gates Reset to
defaults** — the one exception to instant apply.

**Import/export:** JSON carrying `schemaVersion`, `exportedAt` and `appVersion`. Workspace
State is structurally excluded. A newer schema is refused with an explanation rather than
guessed at; an older one is migrated then shown as a preview — *"this will change 4 settings"*
— before applying. Any invalid field rejects the whole file atomically. The current file is
copied to `settings.backup.json` first, and **the import aborts if that backup cannot be
written**, because the backup is the undo.

### Phase 6 — Onboarding

Four full-screen steps on first run: Welcome · Appearance (live preview, applied as you click)
· Privacy (the zero-network promise plus the release-check toggle) · Ready (the shortcuts
worth knowing). Skippable; Defaults apply if skipped.

Progress is `workspace.onboarding.completedVersion: string | null`, **not a boolean**, so a
future major release can replay a single "what's new" step. Replayable from Settings → About.
Living in Workspace State means importing someone else's settings never skips your onboarding.

**It does not trap focus.** It is a route, not a modal — a trap would lock the user away from
the Window Controls, unable to minimise or close during setup. Focus starts on the step
heading; progress dots use `aria-current="step"`; Skip is a real focusable control.

If Settings cannot be written here, finish onboarding anyway and show the Ephemeral Mode
banner — otherwise setup replays on every launch.

### Phase 7 — Errors, reporting and accessibility

**Four error layers.** Root (crash screen, **inline styles only**, so a failure in the theme
layer doesn't take it down); route (`errorComponent` with **`reset`**, keeping Title Bar and
Sidebar alive so the user can navigate away); section (one settings panel fails alone); and
global capture. Global capture means `window.onerror`, `unhandledrejection`, the Rust panic
hook, **explicit logging inside every boundary**, and React 19's `onUncaughtError`,
`onCaughtError` and `onRecoverableError` — a boundary catches its error, so it never reaches
`window.onerror`, and a design relying on that alone reports nothing.

**Reporting**, per `docs/error-reporting.md`: the Diagnostics Bundle with its manifest and a
**preview step before saving**; **Copy summary** as the low-friction default and the zip as
escalation; the correlation id surfaced in all five places; Recent problems in Advanced.

**Shortcuts:** `Ctrl+K` `Ctrl+B` `Ctrl+,` `Ctrl+1…4` `Ctrl+/` `Esc`, plus `Ctrl +/-/0` zoom
enabled explicitly. **`F5`, `Ctrl+R` and devtools are release-disabled** — a reload blanks the
window and is indistinguishable from a crash, and shipping devtools contradicts disabling
reload for the same reason. `Ctrl+/`'s shortcut sheet is also reachable from About, since a
keyboard-only route to the keyboard shortcuts helps nobody who doesn't already know it.

**Accessibility**, the full contract in `docs/accessibility.md`. Division of labour:

| Checked by | Covers |
| --- | --- |
| axe-core (jsdom), per route | Roles, names, labels, ARIA, landmarks, heading order |
| `check-contrast.ts` | Every colour pair, three schemes |
| Manual, per `docs/development.md` | 200% scaling · High Contrast · no-mouse pass · NVDA or Narrator |

**axe in jsdom cannot check contrast** — jsdom computes no layout, the rule no-ops, and the
run comes back green. Focus moves on navigation to *the heading that changed*: the `<h2>`
within Settings, the `<h1>` elsewhere. Focus rings use `outline`, never `box-shadow`, which
High Contrast strips.

### Phase 8 — Release pipeline

Conventional Commits + `release-please`, with **`extra-files` covering `Cargo.toml`,
`tauri.conf.json` and `Cargo.lock`** — the version lives in four places and without this the
About page confidently displays the wrong one.

Merging to `main` opens a release PR with a generated `CHANGELOG.md`; merging that tags,
builds and publishes:

- the **NSIS installer** (per-user, no admin — which matters for an unsigned app),
- a **portable `.zip`** assembled by an explicit step, containing the executable and a
  `wgm.portable` marker (`targets: ["nsis"]` produces no zip),
- **`wgm.pdb`**, so any panic backtrace from any release can be symbolised offline,
- **`SHA256SUMS.txt`**, the integrity story that signing does not yet provide.

The signing step exists and no-ops without a key. `licenses.yml` regenerates
`THIRD-PARTY-LICENSES.md` from `cargo about` and `pnpm licenses`, including the OFL text from
`licenses/`. Issue templates ask for **Copy summary** by name, and for a bundle only when a
maintainer requests one.

---

## 5. Explicitly out of scope

No winget invocation of any kind — no process spawning, no output parsing, no capability
detection. No Package data, no installs, no Kit contents, no `.wgmkit` reader. No end-to-end
tests (`tauri-driver` + WebdriverIO is the eventual route). No code signing. No auto-update.
No second language. No macOS or Linux support.

---

## 6. Revision log

History, not requirements. Where this disagrees with the body above, the body wins.

**Revision 2** — adversarial review. `WM_GETMINMAXINFO`; release check moved to Rust (the CSP
forbade it); `plugin-window-state` dropped (portable-mode split brain); `plugin-autostart`
added; boot failsafe; writability probe; `release-please` `extra-files`; devtools behind a
feature; Settings split from Workspace State; corrected the axe-contrast claim; portable zip
build step; `rust-toolchain.toml`, `packageManager`, Biome generated-file ignores;
`react-compiler-healthcheck`; Vite plugin ordering; font `preload` and the OFL obligation;
Geist Mono deferred; state primitives; inline-styled crash screen; `debug_panic`;
reduced-motion attribute; Rail below 900px; capabilities enumerated; CI split by runner cost.

**Revision 3** — UI/UX review; `docs/design.md` added. Settings five pages → four; real empty
states instead of "coming soon"; contrast gate extended to the neutral ramp; Darker drawn
rather than derived; active nav given a shape cue; settings rows capped at 640px; jargon
replaced in user-facing labels; motion budget set to one orchestrated moment.

**Revision 4** — accessibility and failure-mode review; `docs/accessibility.md` and
`docs/failure-modes.md` added. Static fallback markup in `index.html`; Ephemeral Mode; Window
Controls back to DOM order; onboarding focus trap removed; `outline` not `box-shadow`; `<h2>`
focus inside Settings; 10px resizer with the full splitter pattern; rollbacks announced;
redaction stopped pattern-matching `C:\Users\`; correlation id displayed; Windows file-locking
retry; one write lock; migration failure treated as corruption; per-route window title.

**Revision 5** — error-reporting review; `docs/error-reporting.md` added. `debug = 1` and a
published `.pdb`; boundaries log explicitly and React 19's three root handlers are wired;
source maps shipped; a preview step before saving a bundle; environment variables never
collected; `#[redact]` per field; the Trace ring buffer; Copy summary as the default path;
fixed log format, session banner, flush policy, bundle manifest, and a CI assertion that no
bundle contains the username.

**Revision 6** — consistency pass over the whole document set. Revisions 2–5 appended findings
to this log without editing the body, leaving the body describing superseded behaviour in
**five** places a reader would have implemented wrongly: Window Controls "last in DOM order",
a 6px resizer hit area, a focus-trapped onboarding, focus moving to the `<h1>` inside
Settings, and "All five pages" sitting immediately above "Four pages, not five". The body is
now authoritative and this log is explicitly historical. Also fixed:

- `strip = true` added to the defect list and the whole `[profile.release]` block written out;
  Revision 5 named the problem but never showed the fix.
- **`tauri-plugin-clipboard-manager` was missing** — Copy summary and Copy diagnostics, both
  load-bearing in Revision 5, had no way to reach the clipboard. Capability added too.
- **`tsx` was missing** — `check-contrast.ts` had no runner.
- Phase 0's *done-when* used a `git grep` that matches this plan document, so it could never
  pass; scoped away from `docs/`.
- The four review documents, `index.html`, `public/`, `settings.index.tsx` and the
  `release-please` config files were absent from the file tree.
- The Diagnostics Bundle description in Phase 5 still described Revision 1's contents and
  redaction; reporting now lives in Phase 7 as one subject.
- **i18n moved from Phase 7 to Phase 3.** Phases 4 and 5 write user-facing strings; a catalog
  introduced after them is the retrofit it existed to prevent.
- Ephemeral Mode named consistently (Phase 2 previously said "read-only", a second term for
  the same state); `debug_error`, the ring buffer, session banner and flush policy given a
  home in Phase 2; `allow-set-title` added for the per-route window title.
- Phase 0 exempted from a green gate that runs scripts Phase 1 creates.
