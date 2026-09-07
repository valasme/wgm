/**
 * The English message catalog. **The only place in wgm where user-facing words live.**
 *
 * Rust sends machine codes and structured context across the IPC boundary and never a
 * sentence, so every string a user can read is here. Keys are dotted and flat rather
 * than nested, because `t()` derives its interpolation parameter types from the string
 * literal at each key and a nested object would erase them.
 *
 * Copy is fixed in docs/design.md §8. Two rules decide most of it:
 *
 * - **An empty screen is an invitation to act, not a status report.** Every route in
 *   the scaffold is empty, so the empty states *are* the product right now.
 * - **Buttons name what happens and keep the same word afterwards.** *Export settings*
 *   produces *Settings exported*.
 *
 * Errors state what happened and what to do, and do not apologise.
 */

export const en = {
  // --- The application ---------------------------------------------------
  "app.name": "wgm",
  "app.windowTitle": "{page} · wgm",

  // --- Chrome ------------------------------------------------------------
  "chrome.skipToContent": "Skip to content",
  "chrome.toggleSidebar": "Toggle sidebar",
  "chrome.showSidebar": "Show sidebar",
  "chrome.hideSidebar": "Hide sidebar",
  "chrome.windowControls": "Window controls",
  "chrome.minimize": "Minimise",
  "chrome.maximize": "Maximise",
  "chrome.restore": "Restore",
  "chrome.close": "Close",
  "chrome.sidebarNav": "Main",
  "chrome.sidebarWidth": "Sidebar width",
  "chrome.sidebarWidthValue": "{width} pixels",

  // --- Navigation --------------------------------------------------------
  "nav.search": "Search",
  "nav.kits": "Kits",
  "nav.installed": "Installed",
  "nav.updates": "Updates",
  "nav.settings": "Settings",

  // --- Command palette ---------------------------------------------------
  "palette.open": "Search",
  "palette.placeholder": "Search wgm…",
  "palette.empty": "Nothing matches that.",
  "palette.groupNavigate": "Go to",
  "palette.groupAppearance": "Appearance",
  "palette.groupSettings": "Settings",
  "palette.setColorScheme": "Colour scheme: {scheme}",

  // --- Route: search -----------------------------------------------------
  "route.search.title": "Search",
  "route.search.heading": "Find a package",
  "route.search.body": "Search winget for tools, runtimes and editors.",
  "route.search.fieldLabel": "Search packages",
  "route.search.fieldPlaceholder": "Search packages…",

  // --- Route: kits -------------------------------------------------------
  "route.kits.title": "Kits",
  "route.kits.heading": "No kits yet",
  "route.kits.body":
    "A kit is a set of packages you install together — a language runtime, its tooling and your editor.",
  "route.kits.action": "Create a kit",

  "route.kit.title": "Kit",
  "route.kit.heading": "Kit not found",
  "route.kit.body": "This kit isn't installed and isn't in the catalogue.",
  "route.kit.action": "Back to kits",

  // --- Route: installed --------------------------------------------------
  "route.installed.title": "Installed",
  "route.installed.heading": "Nothing installed through wgm",
  "route.installed.body":
    "Packages you install here will be listed, with their versions and sources.",
  "route.installed.action": "Find a package",

  // --- Route: updates ----------------------------------------------------
  "route.updates.title": "Updates",
  "route.updates.heading": "Everything's up to date",
  "route.updates.body": "wgm checks installed packages against winget when you open this page.",
  "route.updates.action": "Check again",

  // --- Route: not found --------------------------------------------------
  "route.notFound.title": "Not found",
  "route.notFound.heading": "That page doesn't exist",
  "route.notFound.action": "Go to search",

  // --- State primitives --------------------------------------------------
  "state.loading": "Loading…",
  "state.retry": "Try again",

  // --- Settings ----------------------------------------------------------
  "settings.title": "Settings",

  "settings.appearance.title": "Appearance",
  "settings.appearance.colorScheme": "Colour scheme",
  "settings.appearance.colorScheme.description":
    "System follows Windows, and never picks Darker.",
  "settings.appearance.colorScheme.system": "System",
  "settings.appearance.colorScheme.light": "Light",
  "settings.appearance.colorScheme.dark": "Dark",
  "settings.appearance.colorScheme.darker": "Darker",
  "settings.appearance.accent": "Accent",
  "settings.appearance.accent.description":
    "Marks the focused, the selected and the primary action.",
  "settings.appearance.accent.blue": "Blue",
  "settings.appearance.accent.violet": "Violet",
  "settings.appearance.accent.cyan": "Cyan",
  "settings.appearance.accent.emerald": "Emerald",
  "settings.appearance.accent.amber": "Amber",
  "settings.appearance.accent.orange": "Orange",
  "settings.appearance.accent.rose": "Rose",
  "settings.appearance.accent.neutral": "Neutral",
  "settings.appearance.density": "Density",
  "settings.appearance.density.description": "Changes spacing, never text size.",
  "settings.appearance.density.comfortable": "Comfortable",
  "settings.appearance.density.compact": "Compact",
  "settings.appearance.reduceMotion": "Reduce motion",
  "settings.appearance.reduceMotion.description":
    "Overrides the Windows setting in both directions.",
  "settings.appearance.reduceMotion.system": "Follow Windows",
  "settings.appearance.reduceMotion.on": "On",
  "settings.appearance.reduceMotion.off": "Off",

  "settings.general.title": "General",
  "settings.general.restoreWindowPosition": "Restore window position",
  "settings.general.restoreWindowPosition.description":
    "Reopen at the size and place you left it.",
  "settings.general.confirmDestructive": "Confirm before destructive actions",
  "settings.general.confirmDestructive.description":
    "Ask before anything that cannot be undone.",

  "settings.advanced.title": "Advanced",
  "settings.advanced.logLevel": "Log level",
  "settings.advanced.logLevel.description": "Takes effect the next time wgm starts.",
  "settings.advanced.logLevel.error": "Errors only",
  "settings.advanced.logLevel.warn": "Warnings",
  "settings.advanced.logLevel.info": "Normal",
  "settings.advanced.logLevel.debug": "Detailed",
  "settings.advanced.logLevel.trace": "Everything",
  "settings.advanced.logRetention": "Keep logs for",
  "settings.advanced.logRetention.description":
    "Older files are deleted, and so are the oldest once they pass a size limit.",
  "settings.advanced.logRetention.days": "{days} days",
  "settings.advanced.openLogFolder": "Open log folder",
  "settings.advanced.exportDiagnostics": "Export diagnostics",
  "settings.advanced.exportDiagnostics.description":
    "A zip you can attach to a bug report. You'll see everything in it first.",
  "settings.advanced.recentProblems": "Recent problems",
  "settings.advanced.recentProblems.empty": "Nothing has gone wrong this session.",
  "settings.advanced.recentProblems.copy": "Copy",
  "settings.advanced.importSettings": "Import settings",
  "settings.advanced.exportSettings": "Export settings",
  "settings.advanced.reset": "Reset to defaults",
  "settings.advanced.reset.description": "Puts every setting back the way it shipped.",

  "settings.about.title": "About",
  "settings.about.version": "Version",
  "settings.about.commit": "Commit",
  "settings.about.buildDate": "Built",
  "settings.about.tauri": "Tauri",
  "settings.about.webview": "WebView2",
  "settings.about.dataDirectory": "Data folder",
  "settings.about.openDataFolder": "Open folder",
  "settings.about.storageMode.PORTABLE": "Portable",
  "settings.about.storageMode.APP_DATA": "Installed",
  "settings.about.storageMode.EPHEMERAL": "Not saving",
  "settings.about.licences": "Licences",
  "settings.about.checkForRelease": "Check for a new version",
  "settings.about.showSetupAgain": "Show setup again",
  "settings.about.shortcuts": "Keyboard shortcuts",
  "settings.about.disclaimer":
    "wgm is not affiliated with or endorsed by Microsoft. WinGet is a trademark of Microsoft Corporation.",

  // --- Instant apply -----------------------------------------------------
  "settings.rollback": "That didn't save, so it's been put back.",
  "settings.saved": "Saved",

  // --- Import and export -------------------------------------------------
  "import.title": "Import settings",
  "import.preview": "This will change {count} settings.",
  "import.previewOne": "This will change 1 setting.",
  "import.noChanges": "That file matches your current settings.",
  "import.apply": "Apply",
  "import.cancel": "Cancel",
  "import.backupNote": "Your current settings are copied to settings.backup.json first.",
  "export.done": "Settings exported",
  "import.done": "Settings imported",

  // --- Reset -------------------------------------------------------------
  "reset.title": "Reset to defaults?",
  "reset.body":
    "Every setting goes back the way it shipped. Your kits and installs are untouched.",
  "reset.confirm": "Reset to defaults",
  "reset.cancel": "Cancel",
  "reset.done": "Settings reset",

  // --- Diagnostics -------------------------------------------------------
  "diagnostics.title": "Export diagnostics",
  "diagnostics.intro": "Here's everything the file will contain. Nothing is sent anywhere.",
  "diagnostics.publicWarning":
    "Attachments on a GitHub issue are public and permanent — the link outlives the issue.",
  "diagnostics.totalSize": "{count} files, {size}",
  "diagnostics.view": "View",
  "diagnostics.save": "Save",
  "diagnostics.cancel": "Cancel",
  "diagnostics.truncated": "truncated",
  "diagnostics.done": "Diagnostics exported",
  "diagnostics.copySummary": "Copy summary",
  "diagnostics.copySummary.done": "Summary copied",
  "diagnostics.unsafe":
    "wgm couldn't work out which parts of these files to redact, so it hasn't written them.",

  // --- The wgm Release check ---------------------------------------------
  "release.checking": "Checking…",
  "release.upToDate": "You're on the latest version.",
  "release.newer": "Version {version} is available.",
  "release.open": "Open the release page",

  // --- Banners -----------------------------------------------------------
  "banner.ephemeral.title": "Changes aren't being saved",
  "banner.ephemeral.body":
    "wgm can't write to its data folder, so anything you change here lasts until you close it.",
  "banner.corrupt.title": "Settings couldn't be read, so wgm started with defaults.",
  "banner.corrupt.body": "Your previous file is saved as {backup}.",
  "banner.corrupt.action": "Open folder",
  "banner.schemaTooNew.title": "These settings came from a newer version of wgm",
  "banner.schemaTooNew.body":
    "wgm started with defaults and won't overwrite the file. Install the newer version to use it again.",
  "banner.writeFailing.title": "wgm can't save your settings",
  "banner.writeFailing.body":
    "Something else is holding the file open — antivirus and cloud sync both do this. wgm keeps trying.",
  "banner.dismiss": "Dismiss",

  // --- Onboarding --------------------------------------------------------
  "onboarding.skip": "Skip setup",
  "onboarding.back": "Back",
  "onboarding.next": "Next",
  "onboarding.finish": "Get started",
  "onboarding.progress": "Step {step} of {total}",

  "onboarding.welcome.title": "wgm",
  "onboarding.welcome.body":
    "An opinionated winget package manager. Install your tools, group them into kits, and set up the next machine in a minute rather than a morning.",

  "onboarding.appearance.title": "Make it yours",
  "onboarding.appearance.body":
    "Pick a colour scheme and an accent. You can change both later.",

  "onboarding.privacy.title": "wgm sends nothing",
  "onboarding.privacy.body":
    "No telemetry, no analytics, no crash reporting, no auto-update. The only outbound request in the whole product is the version check below, and only when you click it.",
  "onboarding.privacy.releaseCheck": "Offer to check for new versions",
  "onboarding.privacy.releaseCheck.description":
    "Adds a button to Settings → About. Nothing is checked in the background.",

  "onboarding.ready.title": "Worth knowing",
  "onboarding.ready.body": "Four shortcuts and you'll rarely reach for the mouse.",

  // --- Keyboard shortcuts ------------------------------------------------
  "shortcuts.title": "Keyboard shortcuts",
  "shortcuts.palette": "Open the command palette",
  "shortcuts.sidebar": "Show or hide the sidebar",
  "shortcuts.settings": "Open settings",
  "shortcuts.navigate": "Go to the first four pages",
  "shortcuts.help": "Show this list",
  "shortcuts.close": "Close a dialog or the palette",
  "shortcuts.zoom": "Zoom in, out, and back to 100%",
  "shortcuts.systemMenu": "Open the window menu",

  // --- Error boundaries --------------------------------------------------
  "crash.title": "wgm has stopped working",
  "crash.body":
    "Something went wrong that wgm couldn't recover from. Copying the diagnostics below gives a maintainer everything they need.",
  "crash.correlationId": "Reference {id}",
  "crash.copyDiagnostics": "Copy diagnostics",
  "crash.copied": "Copied",
  "crash.openLogFolder": "Open log folder",
  "crash.restart": "Restart wgm",

  "routeError.title": "This page didn't load",
  "routeError.body":
    "The rest of wgm is still working — you can go somewhere else and come back.",
  "routeError.retry": "Try again",

  "sectionError.title": "This section didn't load",
  "sectionError.retry": "Try again",

  // --- Errors, by code ---------------------------------------------------
  // Keyed by the `ErrorCode` Rust sends. `tryTranslate` falls back to the generic
  // message below when a code from a newer build arrives.
  "errors.generic": "Something didn't work.",
  "errors.STORAGE_UNAVAILABLE": "wgm has nowhere it can save to, so changes won't stick.",
  "errors.DOCUMENT_READ_FAILED": "wgm couldn't read {document}.",
  "errors.DOCUMENT_WRITE_FAILED": "wgm couldn't save {document}.",
  "errors.DOCUMENT_CORRUPT":
    "{document} couldn't be read. Your previous file is saved as {backup}.",
  "errors.DOCUMENT_SCHEMA_TOO_NEW":
    "{document} came from a newer version of wgm. It's been left alone.",
  "errors.DOCUMENT_MIGRATION_FAILED": "{document} couldn't be brought up to date.",
  "errors.STORAGE_EPHEMERAL": "Changes aren't being saved.",
  "errors.IMPORT_INVALID": "That file isn't a wgm settings export.",
  "errors.IMPORT_SCHEMA_TOO_NEW":
    "That file came from wgm {found}, and this is version {expected}.",
  "errors.IMPORT_BACKUP_FAILED":
    "wgm couldn't back up your current settings, so it stopped before importing.",
  "errors.EXPORT_FAILED": "wgm couldn't write to {path}.",
  "errors.DIAGNOSTICS_BUNDLE_FAILED": "wgm couldn't build the diagnostics file.",
  "errors.DIAGNOSTICS_REDACTION_UNSAFE":
    "wgm couldn't work out what to redact, so it hasn't written anything.",
  "errors.RELEASE_CHECK_OFFLINE": "Couldn't reach GitHub. Check your connection and try again.",
  "errors.RELEASE_CHECK_SERVER_ERROR": "GitHub isn't responding right now. Try again later.",
  "errors.RELEASE_CHECK_RATE_LIMITED": "GitHub has had too many requests from this network.",
  "errors.RELEASE_CHECK_TIMEOUT": "GitHub took too long to answer.",
  "errors.RELEASE_CHECK_MALFORMED": "GitHub sent something wgm didn't understand.",
  "errors.WINDOW_OPERATION_FAILED": "Windows refused that.",
  "errors.IPC_UNKNOWN": "Something went wrong inside wgm.",
  "errors.DEBUG_FORCED": "This error was triggered on purpose.",

  // --- Path notes, from paths.rs -----------------------------------------
  "pathNote.PORTABLE_FOLDER_READ_ONLY":
    "wgm is running portably but can't write beside itself, so it's using your AppData folder.",
  "pathNote.APP_DATA_UNAVAILABLE": "Windows didn't tell wgm where your AppData folder is.",
  "pathNote.APP_DATA_READ_ONLY": "wgm can't write to your AppData folder.",
} as const;
