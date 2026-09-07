# wgm — Development

Setup lives in [`CONTRIBUTING.md`](../CONTRIBUTING.md). This document is the other
half: **what CI cannot check, and how to check it by hand.**

---

## 1. The gate

Every phase in `docs/PLAN.md` ends green on all four:

```bash
pnpm check && pnpm typecheck && pnpm test
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
```

Lefthook runs Biome, `cargo fmt --check` and Clippy on staged files at commit, and
typecheck plus tests at push. CI re-runs everything across the whole tree, split by
runner cost: the frontend job on `ubuntu-latest`, anything needing Rust or a bundle on
`windows-latest`.

## 2. What is automated, and what it actually proves

| Checked by | Covers | Does **not** cover |
| --- | --- | --- |
| `biome ci` | Format, lint, import order, a11y lint rules | Anything at runtime |
| `tsc` | Types across `src` and the config/scripts project | The IPC boundary at runtime |
| `vitest` + axe-core | Roles, accessible names, ARIA validity, landmarks, heading order | **Colour contrast** — see below |
| `scripts/check-contrast.ts` | Every colour pair, three Color Schemes, WCAG AA | Whether the pair is used where you think |
| `react-compiler-healthcheck` | Components silently bailing out of compilation | Whether the memoisation helps |
| `cargo test` | Document load/migrate/corrupt paths, redaction goldens | Real filesystem behaviour under antivirus |
| `size-limit` | Bundle growth | — |

**axe in jsdom cannot check colour contrast.** jsdom computes no layout, so the rule
silently no-ops and the run comes back green. Reading a passing axe run as contrast
coverage is the specific mistake this table exists to prevent; `check-contrast.ts` is
the only thing that verifies the palette.

## 3. Manual passes, per release

None of these can run in CI. Do all four before tagging, on a real Windows machine.

### 3.1 Display scaling — 200%

Settings → System → Display → Scale → 200%, then sign out and back in.

- [ ] No text is clipped, in any route, in either density.
- [ ] The Title Bar is still 36px in layout terms and the Window Controls are still
      hit-able.
- [ ] The Sidebar resizer still lands within its 10px hit area.
- [ ] No fixed-height container cuts off a descender.

Also try 125% and 150% — the fractional ones are where half-pixel borders disappear.

### 3.2 Windows High Contrast

Settings → Accessibility → Contrast themes → Aquatic, and again with Desert.

- [ ] Every focus ring is still visible. Rings use `outline`, never `box-shadow`,
      because `forced-colors: active` strips box-shadows entirely.
- [ ] The active navigation item is still distinguishable — its indicator must be a
      `border`/`outline`, not a `background`, because backgrounds are overridden.
- [ ] The Window Control glyphs are still drawn. They use `currentColor`; a hardcoded
      fill disappears here.
- [ ] Nothing relies on an Accent. Accents do not survive High Contrast and should not
      try to.

### 3.3 No-mouse pass

Unplug the mouse. Do not touch the trackpad.

- [ ] Tab order is: skip link → sidebar toggle → Window Controls (one stop) → Sidebar
      nav → main. DOM order matches visual order.
- [ ] Arrow keys move between minimise, maximise and close within the Window Controls
      toolbar.
- [ ] The Sidebar resizer moves in 16px steps with the arrow keys; Home and End jump
      to 180 and 400.
- [ ] `Ctrl+K` opens the Command Palette and `Esc` closes it, returning focus.
- [ ] `Ctrl+B`, `Ctrl+,`, `Ctrl+1…4` and `Ctrl+/` all work.
- [ ] Every settings control is reachable and operable, including Reset to defaults
      and its confirmation.
- [ ] Onboarding completes end to end, and Skip is reachable. It must **not** trap
      focus — you can still Tab to the Window Controls and close the app.
- [ ] `Alt+Space` opens the real Windows system menu.

### 3.4 Screen reader — NVDA or Narrator

- [ ] The OS window title changes on navigation: `Settings · wgm`, `Search · wgm`.
      Confirm with Alt+Tab as well as in-app.
- [ ] Focus moves on navigation to *the heading that changed* — the `<h2>` inside
      Settings, the `<h1>` everywhere else.
- [ ] Rail items announce their `aria-label`, not their tooltip.
- [ ] Progress dots in onboarding announce the current step.
- [ ] **Instant-apply rollback is announced.** Toggle something, force the write to
      fail (see §4), and confirm all three: the control returns to its previous state,
      an inline message appears on that row and is read via `aria-describedby`, and a
      toast fires.

## 4. Forcing the failure paths

The reporting path is the code that runs when everything else has failed, so it cannot
be the code that is never exercised. Dev builds carry two commands behind
`#[cfg(debug_assertions)]`:

| Command | Exercises |
| --- | --- |
| `debug_panic` | The Rust panic hook, backtrace capture, log flush, crash screen |
| `debug_error` | `AppError` construction, logging at construction, the correlation id reaching the UI |

Other failure modes, and how to produce them by hand:

| To test | Do this |
| --- | --- |
| Corrupt Settings File | Write `{` into `settings.json` and relaunch. Expect a `settings.corrupt-<ts>.json` copy, a boot from Defaults, and a persistent banner naming the file. The original must never be deleted. |
| Newer `schemaVersion` | Set `schemaVersion` to a large number. Expect Ephemeral Mode and **no overwrite** of the file. |
| Locked file | Open `settings.json` in an application that takes an exclusive lock, then toggle a setting repeatedly. Expect backoff, and **one** coalesced banner — not one toast per toggle. |
| Ephemeral Mode | Mark the data directory read-only. wgm must still start, with the banner. |
| Portable under `Program Files` | Put a `wgm.portable` marker beside the exe in a read-only folder. Expect a fall back to `%APPDATA%`, a logged warning, and the resolved path shown in About. |
| Blank-window boot failure | Break the bundle (rename `dist/assets`). The static fallback markup in `index.html` must appear — never a blank white window. |
| Release-check failures | Disconnect the network for the offline path; the 403 rate-limit path needs 60 requests in an hour from one IP. |

## 5. Checking a Diagnostics Bundle by hand

Export one from Settings → Advanced, then before attaching it anywhere:

- [ ] Search every file in the zip for your Windows username. Zero hits. CI asserts
      this too, but do it once by eye on a real machine — CI runs as `runneradmin`,
      which is not a name a redaction bug would trip over.
- [ ] `manifest.txt` names the version, commit SHA, build id, session id and the
      correlation id being reported, and states anything that was truncated.
- [ ] `system.txt` contains no environment variables. They are never collected, not
      filtered — a developer's environment routinely holds `GITHUB_TOKEN`.
- [ ] `COMPUTERNAME` does not appear.
- [ ] The preview dialog listed every file that ended up in the zip.

## 6. Working on the generated files

Two files are generated, committed, and Biome-ignored. Never hand-edit either:

| File | Generated by | Regenerate with |
| --- | --- | --- |
| `src/routeTree.gen.ts` | `@tanstack/router-plugin` | `pnpm dev` or `pnpm build` |
| `src/ipc/bindings.ts` | `tauri-specta` | `cargo test --manifest-path src-tauri/Cargo.toml` |

If a bindings diff appears that you did not intend, a Rust type changed shape. That is
the friction ADR-0001 is buying, not a bug.

## 7. Releasing

Conventional Commits drive `release-please`. Merging to `main` opens a release PR;
merging that PR tags, builds and publishes the NSIS installer, the portable zip,
`wgm.pdb` and `SHA256SUMS.txt`. The version lives in four places and
`release-please-config.json`'s `extra-files` keeps them in step — without it the About
page confidently displays the wrong one.

Before merging a release PR, run every manual pass above.
