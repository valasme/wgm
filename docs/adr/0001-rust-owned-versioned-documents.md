---
status: accepted
---

# Rust owns all persisted state as Versioned Documents

Every JSON file wgm owns — Settings today, Kit Files next — is loaded, validated, migrated,
imported, exported and written by a single generic Rust module (`src-tauri/src/document/`),
and reaches the webview only through typed commands. The webview has no filesystem access
at all: the `fs` plugin is never enabled, and import/export uses `tauri-plugin-dialog` to
obtain a *path* that Rust then reads.

## Considered options

`tauri-plugin-store` was the obvious alternative and is roughly a tenth of the code. It was
rejected because it is an untyped key-value blob with no schema, no `schemaVersion`, and no
migration path: renaming a setting would silently break every existing user's config, and
validating a hand-edited or hostile import file would have to happen in TypeScript, on the
same side of the trust boundary as the file itself.

## Consequences

- Settings changes cost a Rust edit plus regenerated bindings, not just a React edit. This
  is deliberate friction on a schema that is expensive to get wrong.
- Zero-flash theme boot becomes possible: Rust reads Settings *before* the window is
  created, so it can set the native background colour and inject a synchronous init script.
  A frontend-owned store could not do this without a visible flash.
- Persisted state splits into two documents, not one: **Settings** (portable, exportable) and
  **Workspace State** (window geometry, Sidebar width, onboarding progress — never exported).
  This makes the export exclusions structural rather than a list someone forgets to update,
  and gives the module two consumers immediately, which is what proves it is genuinely generic.
- `tauri-plugin-window-state` is therefore not used: it saves to the app config directory with
  no path override, which would put window geometry in `%APPDATA%` while Settings sat in
  `./data/` under portable mode.
- Kits get the whole import/validate/migrate/export mechanism for free as the third consumer.
