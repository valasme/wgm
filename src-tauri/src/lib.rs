//! wgm — an opinionated winget package manager for developers.
//!
//! Boot order matters and is the reason this file reads the way it does:
//!
//! 1. **Resolve the data directory and load Settings before the window exists.** That
//!    is what makes a zero-flash theme boot possible (ADR-0001) and what lets the log
//!    plugin point at the same directory the documents use.
//! 2. **Install the panic hook before anything can panic.**
//! 3. Register plugins, then commands, then the window hooks.
//!
//! wgm always starts. It never refuses to launch over a storage problem — see
//! `docs/failure-modes.md` §1.

pub mod commands;
pub mod diagnostics;
pub mod document;
pub mod error;
pub mod logging;
pub mod paths;
pub mod platform;
pub mod releases;
pub mod ring;
pub mod settings;
pub mod state;
pub mod workspace;

use tauri::Manager;
use tauri_specta::{collect_commands, Builder};

use crate::settings::schema::ColorSchemePreference;
use crate::state::AppState;

/// The command surface, written once.
///
/// Any extra commands are spliced in ahead of the list, as raw tokens — `collect_commands!`
/// will not accept a `path` metavariable, and the caller has to be able to pass none at
/// all. The alternative, calling `.commands()` a second time under
/// `#[cfg(debug_assertions)]`, is a trap: the second call **replaces** the first rather
/// than appending, which silently reduces the bindings to the two debug commands.
macro_rules! wgm_commands {
    ($($extra:tt)*) => {
        collect_commands![
            $($extra)*
            commands::app_info,
            commands::recent_problems,
            commands::open_known_folder,
            commands::diagnostics_summary,
            commands::diagnostics_preview,
            commands::diagnostics_export,
            commands::diagnostics_suggested_name,
            commands::check_for_release,
            commands::set_maximize_button_rect,
            commands::open_system_menu,
            commands::show_window,
            settings::commands::settings_get,
            settings::commands::settings_status,
            settings::commands::settings_set,
            settings::commands::settings_set_color_scheme,
            settings::commands::settings_set_accent,
            settings::commands::settings_set_density,
            settings::commands::settings_set_reduce_motion,
            settings::commands::settings_set_log_level,
            settings::commands::settings_set_log_retention_days,
            settings::commands::settings_set_start_minimized,
            settings::commands::settings_set_restore_window_position,
            settings::commands::settings_set_confirm_destructive_actions,
            settings::commands::settings_set_release_check_enabled,
            settings::commands::settings_set_launch_on_startup,
            settings::commands::settings_reset,
            settings::commands::settings_export,
            settings::commands::settings_export_preview,
            settings::commands::settings_import_preview,
            settings::commands::settings_import_apply,
            workspace::commands::workspace_get,
            workspace::commands::workspace_status,
            workspace::commands::workspace_set_sidebar_width,
            workspace::commands::workspace_set_sidebar_collapsed,
            workspace::commands::workspace_set_window_geometry,
            workspace::commands::workspace_complete_onboarding,
            workspace::commands::workspace_replay_onboarding,
        ]
    };
}

/// Build the tauri-specta builder.
///
/// Exposed so the bindings test can generate `src/ipc/bindings.ts` from exactly the
/// same command list the app registers — a second list would drift.
pub fn specta_builder() -> Builder {
    // Dev-only commands, so the whole reporting path is exercisable on demand rather
    // than theoretical. Compiled out of release builds entirely, and absent from the
    // release bindings as a result.
    #[cfg(debug_assertions)]
    let handlers = wgm_commands![commands::debug_panic, commands::debug_error,];

    #[cfg(not(debug_assertions))]
    let handlers = wgm_commands![];

    Builder::<tauri::Wry>::new().commands(handlers)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Before anything else, so a panic during boot still produces a log line. The
    // hook is the only artefact when Rust panics before the window exists.
    logging::install_panic_hook();

    // Settings are read before the window is created, which is what lets the native
    // background colour and the injected boot script agree with the user's Color
    // Scheme rather than flashing white first.
    let state = AppState::load();
    let settings = state.settings.get();

    // `None` when the log directory could not be written to. Logging must never be the
    // thing that stops wgm from launching, so this is a probe rather than an attempt —
    // see `logging::plugin`.
    let log_dir = state
        .paths
        .logging_available
        .then(|| state.paths.log_dir.clone());
    let data_dir = state.paths.data_dir.clone();
    let storage_mode = format!("{:?}", state.paths.mode);
    let retention_days = settings.advanced.log_retention_days;
    let file_level = logging::FileLevel::from(settings.advanced.log_level);
    let start_minimized = settings.general.start_minimized;

    let specta = specta_builder();

    let app = tauri::Builder::default()
        // One instance. A second launch focuses the window that already exists rather
        // than opening a second one over the same data directory.
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(logging::plugin(log_dir.as_deref(), file_level))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        // tauri-plugin-http is a dependency but is deliberately **not** registered:
        // the release check runs in Rust, and the webview must not be handed an HTTP
        // client. See ADR-0003.
        .invoke_handler(specta.invoke_handler())
        .manage(state)
        .setup(move |app| {
            specta.mount_events(app);

            logging::session_banner(&data_dir, &storage_mode);

            match &log_dir {
                Some(dir) => logging::prune_logs(dir, retention_days),
                // Worth one line: a bundle from this run will have no log files in it,
                // and a maintainer should be able to see why rather than guess.
                None => log::warn!(
                    target: "wgm::logging",
                    "no writable log directory; this session is logging to memory only",
                ),
            }

            if let Some(window) = app.get_webview_window("main") {
                platform::install_window_hooks(&window);
                apply_native_background(&window, settings.appearance.color_scheme);

                // The window is created hidden and shown on first paint, so boot never
                // flashes an unstyled frame. `start_minimized` skips the show entirely.
                if start_minimized {
                    let _ = window.minimize();
                    let _ = window.show();
                }
            }

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building the wgm application");

    app.run(|_app, _event| {});
}

/// Set the window's native background colour to match the Color Scheme.
///
/// This is the half of the zero-flash boot that CSS cannot do: the frame Windows
/// paints before the webview has rendered anything belongs to the OS, and a dark app
/// that flashes a white rectangle on every launch looks broken.
fn apply_native_background(window: &tauri::WebviewWindow, scheme: ColorSchemePreference) {
    use tauri::window::Color;

    // System resolves to Light or Dark and never to Darker; without an OS signal
    // available this early, Dark is the safer guess for the one-frame case because a
    // white flash is far more visible than a dark one.
    let colour = match scheme {
        ColorSchemePreference::Light => Color(252, 252, 252, 255),
        ColorSchemePreference::Dark | ColorSchemePreference::System => Color(38, 38, 41, 255),
        ColorSchemePreference::Darker => Color(0, 0, 0, 255),
    };

    let _ = window.set_background_color(Some(colour));
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Generates `src/ipc/bindings.ts`.
    ///
    /// A test rather than a build script: it runs on `cargo test`, it fails loudly
    /// when the output cannot be written, and it keeps the generated file out of the
    /// build's critical path. `src/ipc/bindings.ts` is committed and Biome-ignored,
    /// and is **never** hand-edited — a diff you did not intend means a Rust type
    /// changed shape, which is the friction ADR-0001 is buying.
    #[test]
    fn bindings_are_generated() {
        use specta_typescript::Typescript;

        let path = std::path::Path::new("../src/ipc/bindings.ts");

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("could not create src/ipc");
        }

        specta_builder()
            .export(
                Typescript::default()
                    .header("// @ts-nocheck\n// GENERATED by tauri-specta. Do not edit by hand.\n"),
                path,
            )
            .expect("failed to export the IPC bindings");

        let generated = std::fs::read_to_string(path).expect("the bindings must exist");
        assert!(
            generated.contains("settingsGet"),
            "the command list did not reach the bindings"
        );
    }
}
