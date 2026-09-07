<!--
Thanks for contributing. CONTRIBUTING.md lists the rules that get a PR sent back;
this is the short version of the same thing.
-->

## What and why

<!-- One paragraph. What changes, and what problem it solves. -->

Closes #

## Checks

- [ ] `pnpm check && pnpm typecheck && pnpm test` passes.
- [ ] `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` passes.
- [ ] Commits follow [Conventional Commits](https://www.conventionalcommits.org/) —
      `release-please` generates the changelog from them, so the prefix decides the
      release.

## If this touches…

<!-- Delete the rows that don't apply. -->

- **A document schema** — the migration and its checked-in fixture in
  `src-tauri/tests/fixtures/` are in this PR.
- **User-facing text** — every string is in `src/i18n/en.ts`, and Rust still sends only
  a machine code across the boundary.
- **Colour** — the value is in `src/theme/tokens.css` and `pnpm check-contrast` passes.
- **UI** — there is an axe test, focus rings use `outline` rather than `box-shadow`, and
  nothing is signalled by colour alone.
- **A new dependency** — say what it is for here. Anything that phones home is out.
- **`src/ipc/bindings.ts` or `src/routeTree.gen.ts`** — regenerated, not hand-edited.

## Manual testing

<!--
Say what you actually did. docs/development.md §3 lists the four passes CI cannot run:
200% scaling, Windows High Contrast, a no-mouse pass, and NVDA or Narrator.
-->
