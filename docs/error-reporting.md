# wgm — Error Reporting

The path from *something broke on a user's machine* to *a maintainer can fix it*. Eight
links, each of which can break silently:

**capture → record → notice → collect → review → transmit → correlate → fix**

wgm sends nothing automatically (ADR-0003), so every one of these links is the user's
deliberate action. That raises the bar: a report only happens if it is easy, and it is only
useful if it is complete.

---

## 1. Capture

### Rust

`AppError`s are logged **at construction**, not at the point they reach the UI — by then the
call site that knows what was being attempted is gone. Each record carries the code, the
structured context and the correlation id.

**Panics need symbols, and the release profile currently destroys them.** `strip = true` with
no debug info means a backtrace is a column of hex addresses. So:

- `debug = 1` in `[profile.release]` — line tables only, producing a `.pdb`. The shipped
  executable barely grows.
- **`wgm.pdb` is published as a release asset**, not bundled into the installer. Users get a
  small download; a maintainer can symbolise a backtrace from any release offline.
- The panic hook calls `Backtrace::force_capture()` explicitly. Users will never set
  `RUST_BACKTRACE=1`, so relying on the environment variable means never getting a backtrace.
- The hook must not itself panic. A double panic aborts with no log at all — the one failure
  that leaves nothing behind.

### Frontend

**A React error boundary does not fire `window.onerror`.** It catches the error, which means
a boundary that only renders a fallback reports nothing. Every boundary logs explicitly.

React 19's root options are the correct hooks and all three are wired:
`onUncaughtError`, `onCaughtError`, and **`onRecoverableError`** — the last catches
concurrent-rendering and hydration faults that no boundary ever sees.

**Source maps ship with the release.** A minified stack trace reading `Xe@index-a3f.js:1:4823`
is not a bug report. This is a decision a closed-source product could not make and wgm can:
the source is public anyway, so there is nothing to protect and a great deal to gain.

`console.warn` and `console.error` are piped into the Rust log stream; `console.log` and
`console.debug` are not, in release.

## 2. Record

**The log line format is fixed**, because a maintainer will grep it and a user may open it in
Notepad:

```
2026-09-07T14:22:31.123+02:00  ERROR  [settings::commands]  a3f9c1  SETTINGS_WRITE_FAILED  path="%USERPROFILE%\AppData\..." msg="..."
```

Human-readable and machine-parseable, in that order of priority. Timestamps are ISO-8601 with
the local offset, and the timezone is restated in `system.txt` — a maintainer comparing a
user's log against their own otherwise loses an hour to arithmetic.

**Every run emits a session banner** — version, commit, OS build, session id. Daily rotation
means several runs share a file, and without a banner there is no way to tell where the run
being reported begins.

**Error-level records flush immediately**, as does the panic hook. A buffered writer loses
the last lines on a crash, and the last lines are the entire point.

### The Trace ring buffer

The default log level is Info, which means **the bug has already happened at the wrong level**.
Asking a user to reproduce at Trace works only for reproducible bugs, and those are the easy
ones.

So: the last 500 Debug/Trace records are held **in a memory ring buffer regardless of the
configured file level**, and written into the bundle as `trace-tail.log`. It costs a fixed
allocation and it is frequently the only record of an intermittent failure.

## 3. Notice

Errors that are logged but never surfaced are never reported. Settings → Advanced gains a
**Recent problems** list: the last ten errors with timestamp, code and correlation id, and a
per-row *Copy*. The audience is developers; showing them the actual errors raises report
quality more than any amount of instructions.

## 4. Collect — the Diagnostics Bundle

Filename `wgm-diagnostics-0.1.0-2026-09-07T1422.zip`. Contents:

| File | Contents |
| --- | --- |
| `manifest.txt` | What is in the bundle, what was truncated, wgm version, commit SHA, build id (for PDB matching), session id, and the correlation id being reported |
| `wgm-*.log` | Rotated log files, newest first, truncated to the size cap |
| `trace-tail.log` | The ring buffer from §2 |
| `system.txt` | Windows build, display scaling, WebView2 version, Tauri version, active Color Scheme, High Contrast state, locale, timezone, **install kind (installed or portable)**, resolved data directory, whether ephemeral mode is active |
| `settings.json` | Redacted Settings dump |
| `corrupt/` | Any `settings.corrupt-*.json` backups — the exact artefact needed to debug a parse failure |

WebView2 version and display scaling are not padding: rendering and layout bugs are almost
always specific to one or the other.

**The bundle is size-capped.** A Trace-level session can outgrow any day-based retention in
hours. Oldest logs are dropped first and the truncation is stated in `manifest.txt`, so a
maintainer knows the log is partial rather than assuming the app went quiet.

## 5. Review before sending — the missing step

This is the most important gap in the design as it stood.

wgm promises that nothing leaves the machine without the user choosing. Handing them an
opaque zip full of their settings and file paths is not that promise kept — it is that
promise asserted.

**Export shows a preview first**: every file with its size, and one click to open the
redacted `system.txt` or `settings.json` before saving. The user is the last line of defence
against a redaction miss, and they cannot be that if they cannot see what they are sending.

The dialog also states plainly that **GitHub issue attachments are public and permanent** —
the asset URL survives the issue being deleted.

## 6. Redaction

Path redaction is described in `docs/failure-modes.md` §4. Two additions specific to
reporting, both of which are how diagnostics tools leak secrets:

- **Environment variables are never collected.** Not filtered, not allow-listed — not
  collected. A developer's environment routinely holds `GITHUB_TOKEN`, `NPM_TOKEN` and
  cloud credentials, and this audience's environment holds more of them than most.
- **`COMPUTERNAME` is redacted.** Machine names are frequently a person's name or an
  employer's.

**Redaction is declared per field, not applied globally.** Every Settings field is
serialisable into a bundle only if it is not marked `#[redact]`. Today no field holds a
secret; the moment one holds a private winget source URL with credentials, a global path
regex sails straight past it. Marking fields costs nothing now and is the difference between
a leak and a non-event later.

## 7. Transmit

Most users will not attach a zip. So the low-friction path is first-class:

**Copy summary** puts a formatted markdown block on the clipboard — wgm version, commit,
Windows build, WebView2 version, error code, correlation id, and the error message. It pastes
straight into a GitHub issue with no file handling at all. This is what the crash screen's
*Copy diagnostics* produces; the zip is the escalation, not the default.

Issue templates ask for the summary by name, and for a bundle only when a maintainer requests
one.

## 8. Correlate

The correlation id appears in **five** places, and a report is only traceable if all five
agree: the UI error, the toast, the Recent problems list, every related log line, and
`manifest.txt`.

A separate **session id** identifies one run, so a maintainer can isolate the relevant
portion of a shared daily log file.

`manifest.txt` records the exact version, commit SHA and build id, so the maintainer checks
out the right source and matches the right `.pdb`.

## 9. Tests

Error reporting is the code that runs when everything else has failed, so it cannot be the
code that is never tested.

- **Golden-file redaction test**: a log fixture containing a username, a profile path, a UNC
  home directory, a `D:\Users` path and a token-shaped string, asserted against expected output.
- **A CI assertion that a generated bundle contains zero occurrences of the current user's
  username**, in any file. This is the check that catches redaction regressions no reviewer
  would spot.
- `debug_panic` and `debug_error` commands behind `#[cfg(debug_assertions)]`, so the whole
  path — capture, record, surface, collect — is exercisable on demand rather than theoretical.
- A test that the panic hook flushes the log before the process ends.
