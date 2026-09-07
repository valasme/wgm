//! Settings commands.
//!
//! Every one returns `Result<T, AppError>`; nothing here ever returns a string the UI
//! would have to display. Instant apply means these are called on every switch flip,
//! so they are cheap and the store handles the rollback.

use tauri::State;

use crate::document::DocumentStatus;
use crate::error::{AppError, AppResult, ErrorCode};
use crate::settings::schema::{
    Accent, ColorSchemePreference, Density, LogLevel, ReduceMotion, Settings,
};
use crate::settings::{self, ImportPreview, SettingsExport};
use crate::state::AppState;

#[tauri::command]
#[specta::specta]
pub fn settings_get(state: State<'_, AppState>) -> Settings {
    state.settings.get()
}

#[tauri::command]
#[specta::specta]
pub fn settings_status(state: State<'_, AppState>) -> DocumentStatus {
    state.settings.status()
}

/// Replace the whole Settings object.
///
/// One command rather than one per field: the schema is validated as a whole, and a
/// per-field command surface would be a second place for the shape to drift.
#[tauri::command]
#[specta::specta]
pub fn settings_set(state: State<'_, AppState>, settings: Settings) -> AppResult<Settings> {
    state.settings.replace(settings)
}

#[tauri::command]
#[specta::specta]
pub fn settings_set_color_scheme(
    state: State<'_, AppState>,
    value: ColorSchemePreference,
) -> AppResult<Settings> {
    state
        .settings
        .update(|settings| settings.appearance.color_scheme = value)
}

#[tauri::command]
#[specta::specta]
pub fn settings_set_accent(state: State<'_, AppState>, value: Accent) -> AppResult<Settings> {
    state
        .settings
        .update(|settings| settings.appearance.accent = value)
}

#[tauri::command]
#[specta::specta]
pub fn settings_set_density(state: State<'_, AppState>, value: Density) -> AppResult<Settings> {
    state
        .settings
        .update(|settings| settings.appearance.density = value)
}

#[tauri::command]
#[specta::specta]
pub fn settings_set_reduce_motion(
    state: State<'_, AppState>,
    value: ReduceMotion,
) -> AppResult<Settings> {
    state
        .settings
        .update(|settings| settings.appearance.reduce_motion = value)
}

#[tauri::command]
#[specta::specta]
pub fn settings_set_log_level(state: State<'_, AppState>, value: LogLevel) -> AppResult<Settings> {
    let updated = state
        .settings
        .update(|settings| settings.advanced.log_level = value)?;
    // The file level changes for the next run; the Trace ring is unaffected, which is
    // the whole point of it existing.
    log::info!(target: "wgm::settings", "log level set to {value:?}; takes effect on restart");
    Ok(updated)
}

#[tauri::command]
#[specta::specta]
pub fn settings_set_log_retention_days(
    state: State<'_, AppState>,
    value: u32,
) -> AppResult<Settings> {
    let clamped = u64::from(value).clamp(
        *settings::schema::LOG_RETENTION_RANGE.start(),
        *settings::schema::LOG_RETENTION_RANGE.end(),
    );
    state
        .settings
        .update(|settings| settings.advanced.log_retention_days = clamped as u32)
}

#[tauri::command]
#[specta::specta]
pub fn settings_set_start_minimized(
    state: State<'_, AppState>,
    value: bool,
) -> AppResult<Settings> {
    state
        .settings
        .update(|settings| settings.general.start_minimized = value)
}

#[tauri::command]
#[specta::specta]
pub fn settings_set_restore_window_position(
    state: State<'_, AppState>,
    value: bool,
) -> AppResult<Settings> {
    state
        .settings
        .update(|settings| settings.general.restore_window_position = value)
}

#[tauri::command]
#[specta::specta]
pub fn settings_set_confirm_destructive_actions(
    state: State<'_, AppState>,
    value: bool,
) -> AppResult<Settings> {
    state
        .settings
        .update(|settings| settings.general.confirm_destructive_actions = value)
}

#[tauri::command]
#[specta::specta]
pub fn settings_set_release_check_enabled(
    state: State<'_, AppState>,
    value: bool,
) -> AppResult<Settings> {
    state
        .settings
        .update(|settings| settings.privacy.release_check_enabled = value)
}

/// Launch on startup. Written through the autostart plugin **and** recorded in
/// Settings, because the registry entry is the truth and the setting is the intent —
/// and a user who removes the entry by hand should see the switch follow.
#[tauri::command]
#[specta::specta]
pub fn settings_set_launch_on_startup(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    value: bool,
) -> AppResult<Settings> {
    use tauri_plugin_autostart::ManagerExt;

    let manager = app.autolaunch();
    let result = if value {
        manager.enable()
    } else {
        manager.disable()
    };

    result.map_err(|error| {
        AppError::new(ErrorCode::AutostartFailed)
            .with("enabled", value)
            .detail(error.to_string())
            .emit()
    })?;

    state
        .settings
        .update(|settings| settings.general.launch_on_startup = value)
}

/// Reset to defaults. The one action gated by confirm-before-destructive-actions,
/// which is enforced in the UI — this command does what it is told.
#[tauri::command]
#[specta::specta]
pub fn settings_reset(state: State<'_, AppState>) -> AppResult<Settings> {
    log::warn!(target: "wgm::settings", "resetting settings to defaults");
    state.settings.replace(Settings::default())
}

/// The JSON an export writes. Rust produces the bytes; the webview never touches a
/// file, only a path from the native picker.
#[tauri::command]
#[specta::specta]
pub fn settings_export(state: State<'_, AppState>, path: String) -> AppResult<String> {
    let envelope = settings::export_envelope(&state.settings.get());

    let bytes = serde_json::to_vec_pretty(&envelope).map_err(|error| {
        AppError::new(ErrorCode::ExportFailed)
            .detail(error.to_string())
            .emit()
    })?;

    let target = std::path::PathBuf::from(&path);
    let _guard = crate::document::atomic::lock();

    crate::document::atomic::write(&target, &bytes).map_err(|error| {
        AppError::new(ErrorCode::ExportFailed)
            .with("path", &path)
            .detail(error.to_string())
            .emit()
    })?;

    Ok(path)
}

/// Read and validate an import file **without applying it**.
///
/// The preview is the point: an older schema is migrated and then shown as
/// *"this will change 4 settings"* before anything happens.
#[tauri::command]
#[specta::specta]
pub fn settings_import_preview(
    state: State<'_, AppState>,
    path: String,
) -> AppResult<ImportPreview> {
    let raw = std::fs::read_to_string(&path).map_err(|error| {
        AppError::new(ErrorCode::ImportInvalid)
            .with("path", &path)
            .detail(error.to_string())
            .emit()
    })?;

    let (incoming, mut preview, _) = settings::validate_import(&raw)?;
    preview.changes = settings::diff(&state.settings.get(), &incoming);

    Ok(preview)
}

/// Apply an import.
///
/// **The backup is written first and the import aborts if it cannot be.** The backup
/// is the undo; without it the import is not safe to perform.
#[tauri::command]
#[specta::specta]
pub fn settings_import_apply(state: State<'_, AppState>, path: String) -> AppResult<Settings> {
    let raw = std::fs::read_to_string(&path).map_err(|error| {
        AppError::new(ErrorCode::ImportInvalid)
            .with("path", &path)
            .detail(error.to_string())
            .emit()
    })?;

    let (incoming, _, _) = settings::validate_import(&raw)?;

    // Held across both steps, so nothing can write between the backup and the apply.
    let _guard = crate::document::atomic::lock();
    crate::document::backup::<Settings>(state.settings.path())?;

    // `replace` takes the same lock, so the backup happens under a lock this thread
    // already holds. std::sync::Mutex is not reentrant, so the write is done directly.
    let applied = incoming.clone();
    crate::document::save_locked(state.settings.path(), &applied)?;
    state.settings.replace_in_memory(applied.clone());

    log::info!(target: "wgm::settings", "settings imported from {path:?}");

    Ok(applied)
}

/// Everything an export envelope carries, for a UI that wants to show it.
#[tauri::command]
#[specta::specta]
pub fn settings_export_preview(state: State<'_, AppState>) -> SettingsExport {
    settings::export_envelope(&state.settings.get())
}
