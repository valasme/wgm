<div align="center">

# wgm

**An opinionated winget package manager for developers.**

</div>

wgm is a Windows desktop app that browses, installs and maintains developer tooling
through [winget](https://learn.microsoft.com/windows/package-manager/), and groups it
into shareable **Kits** — a language runtime, its tooling and your editor, installed
as one unit.

> **Status: scaffold.** This repository currently builds a complete, production-shaped
> application shell with no product features. Settings works end to end; every other
> route renders its real empty state. wgm does not invoke winget yet. See
> [`docs/PLAN.md`](docs/PLAN.md).

## Why

Setting up a development machine is a morning of copy-pasted install commands, and
doing it again on the next machine is the same morning. winget already solves the
installing; what it does not give you is a set you can name, share and reinstall.

## Requirements

- **Windows 10 version 1809 (build 17763) or newer** — the same floor winget itself
  has. Windows 11 works.
- **WebView2 Runtime** — already present on Windows 11 and on current Windows 10. If
  it is missing, wgm will not start;
  [install it from Microsoft](https://developer.microsoft.com/microsoft-edge/webview2/).
- x64. No ARM64 build yet.

macOS and Linux are not supported and are not planned — wgm drives a Windows-only tool.

## Install

Download the latest release from the
[releases page](https://github.com/valasme/wgm/releases). Two artifacts:

| Artifact | Use it when |
| --- | --- |
| `wgm_<version>_x64-setup.exe` | Normal install. Per-user, **no administrator rights needed**. |
| `wgm_<version>_x64-portable.zip` | You want it on a USB stick or in a folder you control. Unzip and run; it keeps its data in `./data/` beside the executable. |

### The SmartScreen warning is expected

wgm is **not code-signed**. A code signing certificate costs a few hundred dollars a
year, and Microsoft SmartScreen distrusts new certificates anyway until enough people
have installed the app to build reputation — so paying does not remove the warning on
day one.

So Windows will show you *"Windows protected your PC"*. That message means Windows has
not seen this file before. It does not mean the file is malicious, and it does not
mean the file is safe either. **Verify the download yourself.**

To run it: click **More info**, then **Run anyway**.

### Verify the download

Every release publishes `SHA256SUMS.txt`. In PowerShell, from your downloads folder:

```powershell
Get-FileHash .\wgm_0.1.0_x64-setup.exe -Algorithm SHA256
```

Compare the printed hash against the line for that file in `SHA256SUMS.txt`. If they
differ, do not run it — open a
[security advisory](https://github.com/valasme/wgm/security/advisories/new).

Until wgm is signed, this checksum is the whole integrity story, which is why it is
published for every artifact.

## Privacy: wgm sends nothing

Not "anonymised telemetry". Nothing.

- No telemetry, no analytics, no crash reporting, no usage counters.
- No auto-updater and no background version polling.
- No web fonts — Geist and Instrument Serif are bundled with the app.
- The Content-Security-Policy blocks the webview from reaching the network at all, so
  this is enforced by the runtime rather than promised in a document.

There is exactly **one** outbound request in the entire product: *Check for a new
version*, in Settings → About, which queries the GitHub Releases API when you click it,
sends no identifiers, and displays a version number and a link. It is written in Rust
and lives in one auditable function.

The consequence you should know about: because there is no auto-update, **a security
fix reaches you only when you install it**. Watch this repository for releases.

Nothing leaves your machine unless you export a Diagnostics Bundle and attach it to an
issue yourself. That bundle redacts your username and profile path, and shows you every
file it contains before it writes the zip.

Details: [`docs/adr/0003-zero-network-policy.md`](docs/adr/0003-zero-network-policy.md).

## Data location

| Install kind | Location |
| --- | --- |
| Installer | `%APPDATA%\io.github.valasme.wgm\` |
| Portable | `./data/` beside the executable |

Settings → About shows the resolved path and a button to open it. If neither location
is writable, wgm still starts — in Ephemeral Mode, with a banner saying changes will
not be saved.

## Building from source

```bash
pnpm install
pnpm tauri dev
```

Full setup and the rules that get PRs sent back: [`CONTRIBUTING.md`](CONTRIBUTING.md).

## Documentation

| | |
| --- | --- |
| [`CONTEXT.md`](CONTEXT.md) | The glossary. What everything is called, and why. |
| [`docs/PLAN.md`](docs/PLAN.md) | The scaffold implementation plan. |
| [`docs/design.md`](docs/design.md) | Palette, type scale, layout rules, every user-facing string. |
| [`docs/accessibility.md`](docs/accessibility.md) | The accessibility contract. |
| [`docs/failure-modes.md`](docs/failure-modes.md) | Every way wgm can fail, and what it does instead. |
| [`docs/error-reporting.md`](docs/error-reporting.md) | How a failure reaches a maintainer. |
| [`docs/adr/`](docs/adr/) | The three decisions that constrain everything. |

## Licence

[MIT](LICENSE), with the wgm name, wordmark and icon carved out of the grant — fork
freely, but ship it under your own name.

## Disclaimer

wgm is not affiliated with, endorsed by, or sponsored by Microsoft Corporation.
WinGet and Windows are trademarks of Microsoft Corporation.
