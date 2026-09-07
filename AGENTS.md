# wgm

An opinionated winget package manager for developers. Windows desktop app built with
Tauri 2 (Rust) + React 19 + TypeScript.

wgm is **not** affiliated with or endorsed by Microsoft. WinGet is a trademark of
Microsoft Corporation.

## Ground rules

- **`wgm` and `winget` are never synonyms.** wgm is this application; winget is the
  Microsoft tool it will eventually drive. Never conflate them in code, comments, or UI copy.
- **Read `CONTEXT.md` before naming anything.** It is the glossary. Use its terms exactly.
- **Zero network by default.** The only outbound request the app may ever make is a
  manual, user-initiated update check. No telemetry, no analytics, no crash reporting,
  no font CDNs. See `docs/adr/0003-zero-network-policy.md`.
- **Rust owns persisted state.** The webview has no filesystem access. All file I/O goes
  through Rust commands. See `docs/adr/0001-rust-owned-versioned-documents.md`.
- **Rust never sends English to the UI.** Errors carry machine codes; the i18n catalog
  at `src/i18n/en.ts` owns every user-facing word.
- **`src/ipc/bindings.ts` is generated** by tauri-specta. Never edit it by hand.

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

## Agent skills

### Issue tracker

Issues live in GitHub Issues for `valasme/wgm`, managed via the `gh` CLI.
See `docs/agents/issue-tracker.md`.

### Triage labels

Default five-role vocabulary, label strings unchanged.
See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: `CONTEXT.md` and `docs/adr/` at the repo root.
See `docs/agents/domain.md`.
