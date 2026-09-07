//! App-level commands: environment, diagnostics, the release check, window behaviour,
//! and the two dev-only failure triggers.

use tauri::{Manager, State};

use crate::diagnostics::{bundle, summary, ClientEnvironment, Redactor, ReportSubject};
use crate::error::{AppError, AppResult, ErrorCode};
use crate::logging;
use crate::paths::{PathNote, StorageMode};
use crate::platform;
use crate::releases::{self, ReleaseCheck};
use crate::ring::ProblemRecord;
use crate::state::AppState;

/// Everything Settings → About displays, in one round trip.
#[derive(Debug, Clone, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub version: String,
    pub commit: String,
    pub build_date: String,
    pub tauri_version: String,
    pub session_id: String,
    /// Already redacted: About is a screen people screenshot.
    pub data_dir: String,
    pub log_dir: String,
    pub storage_mode: StorageMode,
    pub path_notes: Vec<PathNote>,
    pub ephemeral: bool,
}

#[tauri::command]
#[specta::specta]
pub fn app_info(state: State<'_, AppState>) -> AppInfo {
    let redactor = Redactor::from_environment();

    AppInfo {
        version: env!("CARGO_PKG_VERSION").to_owned(),
        commit: logging::build_commit().to_owned(),
        build_date: logging::build_date().to_owned(),
        tauri_version: tauri::VERSION.to_owned(),
        session_id: logging::session_id().to_owned(),
        data_dir: redactor.redact_path(&state.paths.data_dir),
        log_dir: redactor.redact_path(&state.paths.log_dir),
        storage_mode: state.paths.mode,
        path_notes: state.paths.notes.clone(),
        ephemeral: state.is_ephemeral(),
    }
}

/// The last ten failures, for Settings → Advanced → Recent problems.
///
/// Errors that are logged but never surfaced are never reported, and the audience
/// here is developers: showing them the actual errors raises report quality more than
/// any amount of instructions.
#[tauri::command]
#[specta::specta]
pub fn recent_problems() -> Vec<ProblemRecord> {
    crate::ring::recent_problems()
}

/// Open a folder in Explorer. Takes an enum rather than a path, so the webview cannot
/// name an arbitrary directory.
#[derive(Debug, Clone, Copy, serde::Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum KnownFolder {
    Data,
    Logs,
}

#[tauri::command]
#[specta::specta]
pub fn open_known_folder(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    folder: KnownFolder,
) -> AppResult<()> {
    use tauri_plugin_opener::OpenerExt;

    let path = match folder {
        KnownFolder::Data => state.paths.data_dir.clone(),
        KnownFolder::Logs => state.paths.log_dir.clone(),
    };

    // Creating it first means "Open log folder" works on a fresh install that has not
    // logged anything yet, rather than failing with a path that looks wrong.
    let _ = std::fs::create_dir_all(&path);

    app.opener()
        .open_path(path.to_string_lossy(), None::<&str>)
        .map_err(|error| {
            AppError::new(ErrorCode::WindowOperationFailed)
                .with("folder", format!("{folder:?}"))
                .detail(error.to_string())
                .emit()
        })
}

/// **Copy summary** — the low-friction default. Returns the markdown; the webview
/// puts it on the clipboard.
#[tauri::command]
#[specta::specta]
pub fn diagnostics_summary(
    state: State<'_, AppState>,
    client: ClientEnvironment,
    subject: ReportSubject,
) -> String {
    summary::build(
        &state.paths,
        &client,
        &subject,
        &Redactor::from_environment(),
    )
}

/// Build the bundle and describe it **without writing anything**.
#[tauri::command]
#[specta::specta]
pub fn diagnostics_preview(
    state: State<'_, AppState>,
    client: ClientEnvironment,
    subject: ReportSubject,
) -> bundle::BundlePreview {
    bundle::build(&state.paths, &state.settings.get(), &client, &subject).0
}

/// Write the bundle the user has just previewed.
///
/// Rebuilt rather than carried over from the preview call: a few hundred milliseconds
/// of log lines may have been written in between, and the manifest states the sizes of
/// what is actually in the zip.
#[tauri::command]
#[specta::specta]
pub fn diagnostics_export(
    state: State<'_, AppState>,
    path: String,
    client: ClientEnvironment,
    subject: ReportSubject,
) -> AppResult<String> {
    let (preview, staged) = bundle::build(&state.paths, &state.settings.get(), &client, &subject);

    bundle::write(std::path::Path::new(&path), &staged, preview.redaction_safe)?;

    log::info!(target: "wgm::diagnostics", "diagnostics bundle written");

    Ok(path)
}

/// The name the save dialog should suggest.
#[tauri::command]
#[specta::specta]
pub fn diagnostics_suggested_name() -> String {
    bundle::suggested_name()
}

/// The wgm Release check. The only outbound request in the product, and it happens
/// only because the user clicked.
#[tauri::command]
#[specta::specta]
pub async fn check_for_release(state: State<'_, AppState>) -> AppResult<ReleaseCheck> {
    if !state.settings.get().privacy.release_check_enabled {
        // A control that has been switched off must not act. The UI hides the button
        // too; this is the second half of the same rule.
        return Err(AppError::new(ErrorCode::ReleaseCheckOffline)
            .detail("the release check is disabled in Settings")
            .emit());
    }

    releases::check().await
}

/// Report where the maximise button ended up, so `WM_NCHITTEST` can answer
/// `HTMAXBUTTON` over it and Snap Layouts still appears.
#[tauri::command]
#[specta::specta]
pub fn set_maximize_button_rect(rect: platform::ButtonRect) {
    platform::set_maximize_button_rect(rect);
}

/// `Alt+Space`. Undecorated windows lose the system menu by default, and it is how
/// screen reader users have always reached window commands.
#[tauri::command]
#[specta::specta]
pub fn open_system_menu(app: tauri::AppHandle) -> AppResult<()> {
    let Some(window) = app.get_webview_window("main") else {
        return Err(AppError::new(ErrorCode::WindowOperationFailed)
            .detail("no main window")
            .emit());
    };

    platform::open_system_menu(&window).map_err(|reason| {
        AppError::new(ErrorCode::WindowOperationFailed)
            .detail(reason)
            .emit()
    })
}

/// Show the window.
///
/// Called on first paint, and by a 3-second failsafe in the frontend. Either alone is
/// insufficient: the failsafe without the static fallback markup reveals a blank white
/// window, which is the worst failure in the app.
#[tauri::command]
#[specta::specta]
pub fn show_window(app: tauri::AppHandle) -> AppResult<()> {
    let Some(window) = app.get_webview_window("main") else {
        return Err(AppError::new(ErrorCode::WindowOperationFailed)
            .detail("no main window")
            .emit());
    };

    window
        .show()
        .and_then(|()| window.set_focus())
        .map_err(|error| {
            AppError::new(ErrorCode::WindowOperationFailed)
                .detail(error.to_string())
                .emit()
        })
}

// ---------------------------------------------------------------- dev only ---
//
// So the whole reporting path — capture, record, surface, collect — is exercisable on
// demand rather than theoretical. Compiled out of release builds entirely.

/// Force a panic, to exercise the panic hook, backtrace capture, log flush and the
/// crash screen.
#[cfg(debug_assertions)]
#[tauri::command]
#[specta::specta]
pub fn debug_panic() {
    panic!("debug_panic: this is a deliberate panic from the dev-only command");
}

/// Force an `AppError`, to exercise construction-time logging, the Recent problems
/// ring and the correlation id reaching the UI.
#[cfg(debug_assertions)]
#[tauri::command]
#[specta::specta]
pub fn debug_error() -> AppResult<()> {
    Err(AppError::new(ErrorCode::DebugForced)
        .with("source", "debug_error")
        .detail("a deliberate error from the dev-only command")
        .emit())
}
