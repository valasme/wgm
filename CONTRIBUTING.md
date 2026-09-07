# Contributing to wgm

Thanks for looking. wgm is a Windows desktop app built with Tauri 2 (Rust) and
React 19.

## Before you write code

Read these three, in this order. They are short, and they decide most review
outcomes:

1. **[`CONTEXT.md`](CONTEXT.md)** — the glossary. Names in this codebase are not
   negotiable; use the terms exactly as defined. In particular, `wgm` and `winget`
   are never synonyms.
2. **[`docs/adr/`](docs/adr/)** — three decisions that constrain everything:
   Rust owns persisted state, three OKLCH Color Schemes, and zero network.
3. **[`AGENTS.md`](AGENTS.md)** — the ground rules, in one page.

## Setup

Requirements:

- Windows 10 1809 (build 17763) or newer
- [Rust](https://rustup.rs/) — the toolchain is pinned in `rust-toolchain.toml`
- Node.js 22+ and pnpm via corepack (`corepack enable`)
- [WebView2 Runtime](https://developer.microsoft.com/microsoft-edge/webview2/) —
  preinstalled on Windows 11 and current Windows 10

```bash
pnpm install
pnpm tauri dev
```

## Commands

| Task | Command |
| --- | --- |
| Dev | `pnpm tauri dev` |
| Typecheck | `pnpm typecheck` |
| Lint + format | `pnpm check` |
| Unit + a11y tests | `pnpm test` |
| Rust tests | `cargo test --manifest-path src-tauri/Cargo.toml` |
| Rust lint | `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` |
| Build | `pnpm tauri build` |

A pre-commit hook (lefthook) runs Biome, `cargo fmt --check` and Clippy on staged
files. CI runs all of the above plus the contrast gate and the React Compiler
healthcheck.

## Rules that get PRs sent back

These are not style preferences; each one exists because of a decision recorded in
`docs/`.

- **No network requests from the webview.** The only outbound call in wgm is the
  manual wgm Release check, and it lives in `src-tauri/src/releases.rs`. The CSP
  enforces this. No telemetry, no analytics, no crash reporting, no font CDNs.
- **No filesystem access from the webview.** The `fs` plugin is never enabled. All
  I/O goes through a Rust command.
- **Rust never sends English to the UI.** Errors carry a machine `code` and
  structured context; `src/i18n/en.ts` owns every user-facing word.
- **No hardcoded colours.** Every colour is an OKLCH custom property from
  `src/theme/tokens.css`, and it must pass `scripts/check-contrast.ts` in all three
  Color Schemes.
- **Focus rings use `outline`, never `box-shadow`.** Windows High Contrast strips
  box-shadows.
- **Generated files are never hand-edited**: `src/ipc/bindings.ts` (tauri-specta)
  and `src/routeTree.gen.ts` (TanStack Router). Both are Biome-ignored and both are
  committed.
- **No dead controls.** A control that does not do anything yet does not get
  rendered.
- **Adding a dependency needs a reason in the PR description.** Anything that
  phones home is rejected outright.

## Commits

[Conventional Commits](https://www.conventionalcommits.org/). `release-please`
generates the changelog and version bumps from them, so the prefix decides the
release.

```
feat(settings): add log retention control
fix(sidebar): keep resizer reachable below 900px
docs: explain the SmartScreen warning
chore(deps): bump tauri to 2.1
```

`feat!:` or a `BREAKING CHANGE:` footer for anything that changes a document schema
without a migration.

## Pull requests

- One subject per PR.
- A schema change ships with its migration **and** a checked-in fixture in
  `src-tauri/tests/fixtures/`.
- New UI ships with an axe test and every string in `src/i18n/en.ts`.
- Say what you tested manually. `docs/development.md` lists the passes CI cannot
  run — 200% scaling, High Contrast, a no-mouse pass, and NVDA or Narrator.

## Scope

wgm does not invoke winget yet, and that is deliberate — see `docs/PLAN.md` §5.
A PR that adds package installation is out of scope until the scaffold is finished.

## Reporting bugs

Use the issue templates. Settings → Advanced → *Copy summary* puts everything a
maintainer needs on your clipboard; attach a Diagnostics Bundle only if asked.
Security problems go through
[GitHub Security Advisories](SECURITY.md), not the issue tracker.

## Code of Conduct

By participating you agree to the [Code of Conduct](CODE_OF_CONDUCT.md).
