# wgm

An opinionated winget package manager for developers: a Windows desktop app that browses,
installs and maintains developer tooling, and groups it into shareable Kits.

## Language

### The application

**wgm**:
This application. Always lowercase, never capitalised, never expanded to a phrase.
_Avoid_: WGM, Wgm, "the app" in code identifiers

**winget**:
Microsoft's Windows Package Manager CLI, which wgm drives. A separate product with a
separate lifecycle.
_Avoid_: using it interchangeably with `wgm`; "Windows Package Manager" in code

### Packages and Kits

**Package**:
One winget-installable unit, identified by its winget package identifier.
_Avoid_: App, Application, Program, Software

**Kit**:
A curated, shareable set of Packages installed as a unit, such as a Node web-development
setup. A Kit is a file, not a server record.
_Avoid_: Bundle, Profile, Collection, Preset, Group, Pack

**Kit File**:
The on-disk `.wgmkit` JSON representation of a Kit.

**Package Update**:
A newer version of an installed Package, available from winget. What the `/updates` route
is about.

**wgm Release**:
A newer version of wgm itself, published on GitHub. What Settings → About checks for.
_Avoid_: calling this an "update" without qualification — `/updates` already means
Package Updates

### Persisted state

**Versioned Document**:
Any JSON file wgm owns that carries a `schemaVersion` and can be validated, migrated,
imported and exported through one shared mechanism. Settings and Kit Files are both
Versioned Documents.

**Settings**:
The validated, in-memory configuration object holding everything that is meaningful on
*any* machine, and therefore everything that is exportable. Distinct from the file it came
from.
_Avoid_: Preferences, Options, Config (as a name for this object)

**Settings File**:
The on-disk JSON that a Settings object is loaded from and saved to.

**Workspace State**:
The separate Versioned Document holding everything true only of *this* installation on
*this* machine: window geometry, Sidebar width, Rail state, onboarding progress. Never
exported, never imported.
_Avoid_: putting any of these fields in Settings "just for now"

**Defaults**:
The compiled-in baseline Settings. The fallback when a field is missing or a file is
unreadable; never itself persisted as a separate document.

### Appearance

**Appearance**:
The umbrella term covering Color Scheme, Accent and density together.
_Avoid_: Theme (too broad; it has meant all three at different times)

**Color Scheme**:
Which of Light, Dark or Darker is in effect. `System` is a *preference* that resolves to
Light or Dark, never to Darker.
_Avoid_: Theme, Mode, Dark mode

**Accent**:
The single user-selected hue driving primary buttons and interactive emphasis. One of
eight fixed, contrast-verified choices.
_Avoid_: Primary color, Brand color, Highlight

### Application chrome

**Title Bar**:
The custom 36px bar at the top of the window holding the sidebar toggle, the Wordmark and
the Window Controls. Replaces the native Windows title bar.

**Window Controls**:
The minimise, maximise/restore and close buttons. One toolbar, one tab stop.

**Wordmark**:
The `wgm` lettering set in Instrument Serif Italic. Not a logo, not a title.

**Sidebar**:
The expanded, resizable left navigation column.

**Rail**:
The collapsed 52px icon-only form of the Sidebar. The same component in a different state,
not a separate component.
_Avoid_: Mini sidebar, Collapsed nav, Icon bar

**Content Area**:
Everything to the right of the Sidebar and below the Title Bar.

### Diagnostics

**Diagnostics Bundle**:
The exportable `.zip` containing log files, system information and a redacted Settings
dump. What a user attaches to a bug report.
_Avoid_: Log export, Logs, Debug dump

**Redaction**:
The mandatory, non-optional rewriting of user-identifying paths (notably
`C:\Users\<name>` to `%USERPROFILE%`) before anything leaves the machine.

**Correlation Id**:
The identifier shared between an error shown in the UI and the log lines that produced it,
so a user-reported code can be found in a Diagnostics Bundle.

**Session Id**:
The identifier for one run of wgm, emitted in the startup log banner. Distinguishes runs
that share a rotated log file; a Correlation Id identifies one failure within a run.

**Summary**:
The markdown block wgm copies to the clipboard — version, build, error code and Correlation
Id. The low-friction alternative to a Diagnostics Bundle, and what most bug reports will
actually contain.

**Ephemeral Mode**:
The degraded state wgm runs in when it cannot persist: no writable data directory, or a
Settings File written by a newer version that must not be overwritten. Settings live in
memory only and a banner says so. wgm always starts; it never refuses to launch over a
storage problem.
_Avoid_: read-only mode, safe mode, recovery mode — one name for one state
