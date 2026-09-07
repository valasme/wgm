//! Workspace State commands.
//!
//! Nothing here is exportable and nothing here is a setting. Sidebar width is not a
//! settings row — it lives here and is adjusted by dragging.

use tauri::State;

use crate::document::DocumentStatus;
use crate::error::AppResult;
use crate::state::AppState;
use crate::workspace::schema::{WindowGeometry, WorkspaceState, SIDEBAR_WIDTH_RANGE};

#[tauri::command]
#[specta::specta]
pub fn workspace_get(state: State<'_, AppState>) -> WorkspaceState {
    state.workspace.get()
}

#[tauri::command]
#[specta::specta]
pub fn workspace_status(state: State<'_, AppState>) -> DocumentStatus {
    state.workspace.status()
}

/// Persist the Sidebar width.
///
/// The frontend debounces this by 300 ms during a drag: the width updates live on
/// screen and the file is written once the user stops, rather than sixty times a
/// second through a single global write lock.
#[tauri::command]
#[specta::specta]
pub fn workspace_set_sidebar_width(
    state: State<'_, AppState>,
    width: u32,
) -> AppResult<WorkspaceState> {
    let clamped =
        u64::from(width).clamp(*SIDEBAR_WIDTH_RANGE.start(), *SIDEBAR_WIDTH_RANGE.end()) as u32;

    state
        .workspace
        .update(|workspace| workspace.sidebar.width = clamped)
}

/// Persist the Rail state.
///
/// Only a deliberate collapse is recorded. A Sidebar forced to the Rail because the
/// window is under 900px wide is **not** written here — a narrow window must not
/// permanently collapse the Sidebar.
#[tauri::command]
#[specta::specta]
pub fn workspace_set_sidebar_collapsed(
    state: State<'_, AppState>,
    collapsed: bool,
) -> AppResult<WorkspaceState> {
    state
        .workspace
        .update(|workspace| workspace.sidebar.collapsed = collapsed)
}

#[tauri::command]
#[specta::specta]
pub fn workspace_set_window_geometry(
    state: State<'_, AppState>,
    geometry: WindowGeometry,
) -> AppResult<WorkspaceState> {
    state
        .workspace
        .update(|workspace| workspace.window = geometry)
}

/// Record that setup finished, against the version that ran it.
///
/// A version string rather than a boolean, so a future major release can replay a
/// single "what's new" step without replaying the whole thing.
#[tauri::command]
#[specta::specta]
pub fn workspace_complete_onboarding(state: State<'_, AppState>) -> AppResult<WorkspaceState> {
    let version = env!("CARGO_PKG_VERSION").to_owned();

    let result = state
        .workspace
        .update(|workspace| workspace.onboarding.completed_version = Some(version.clone()));

    // If this could not be written, onboarding still finished. Replaying setup on
    // every launch is a maddening symptom for a small failure, and the Ephemeral Mode
    // banner already explains why nothing is being saved.
    if let Err(error) = &result {
        log::warn!(
            target: "wgm::workspace",
            "onboarding completed but could not be persisted ({code})",
            code = error.code,
        );
    }

    Ok(state.workspace.get())
}

/// *Show setup again*, from Settings → About.
#[tauri::command]
#[specta::specta]
pub fn workspace_replay_onboarding(state: State<'_, AppState>) -> AppResult<WorkspaceState> {
    state
        .workspace
        .update(|workspace| workspace.onboarding.completed_version = None)
}
