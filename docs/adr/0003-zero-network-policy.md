---
status: accepted
---

# wgm makes no network request the user did not personally trigger

There is no telemetry, no analytics, no crash reporting, no auto-updater and no background
polling of any kind. The single permitted outbound request is a "Check for updates" button
in Settings → About that queries the GitHub Releases API only when clicked, sends no
identifiers, and does nothing but display a version number and a link. Updates are
downloaded and installed by the user, manually.

That request is made **from Rust**, in `src-tauri/src/releases.rs`, never from the webview.
This is what lets the enforcement be structural rather than conventional: the
Content-Security-Policy in `tauri.conf.json` restricts `connect-src` to `'self'` and `ipc:`,
so the webview *cannot* reach the network even if someone later writes a `fetch` by mistake.
Fonts are self-hosted rather than loaded from a CDN, and `tauri-plugin-updater` is
deliberately absent from `Cargo.toml`.

Every outbound request in wgm therefore lives in exactly one auditable Rust function.

## Consequences

- No privacy policy, no consent flow and no GDPR posture are required, because no personal
  data is ever collected or transmitted.
- The Diagnostics Bundle is the only route by which anything leaves a user's machine, and
  the user chooses when and to whom. Path redaction inside it is therefore mandatory and
  has no opt-out.
- Adding any dependency that phones home — an error reporter, a font CDN, an analytics
  shim — is a violation of this ADR and should be rejected in review, not debated.
