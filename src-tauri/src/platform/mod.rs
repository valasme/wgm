//! Platform-specific behaviour.
//!
//! wgm targets Windows only, so this is a single directory rather than a trait with
//! one implementation. The wrappers below exist so `lib.rs` and the command layer
//! never carry a `#[cfg]`.

#[cfg(windows)]
pub mod windows;

#[cfg(windows)]
pub use windows::{maximize_button_rect, set_maximize_button_rect, ButtonRect};

/// Install the window subclass that restores Snap Layouts and the work-area clamp.
#[cfg(windows)]
pub fn install_window_hooks(window: &tauri::WebviewWindow) {
    if let Err(reason) = windows::window_proc::install(window) {
        // Not fatal. The app is fully usable without it; Snap Layouts on hover and
        // the taskbar clamp are the only casualties, and saying so in the log is more
        // useful than refusing to start.
        log::warn!(target: "wgm::platform", "native window hooks unavailable: {reason}");
    }
}

#[cfg(not(windows))]
pub fn install_window_hooks(_window: &tauri::WebviewWindow) {}

/// Open the real Windows system menu, as `Alt+Space` would on a decorated window.
#[cfg(windows)]
pub fn open_system_menu(window: &tauri::WebviewWindow) -> Result<(), String> {
    windows::system_menu::open(window)
}

#[cfg(not(windows))]
pub fn open_system_menu(_window: &tauri::WebviewWindow) -> Result<(), String> {
    Ok(())
}

// The non-Windows shims exist so `cargo test` runs on a Linux CI runner. wgm does not
// support macOS or Linux and is not going to.
#[cfg(not(windows))]
pub use fallback::{maximize_button_rect, set_maximize_button_rect, ButtonRect};

#[cfg(not(windows))]
mod fallback {
    use serde::{Deserialize, Serialize};
    use specta::Type;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, Type)]
    #[serde(rename_all = "camelCase")]
    pub struct ButtonRect {
        pub x: i32,
        pub y: i32,
        pub width: i32,
        pub height: i32,
    }

    pub fn set_maximize_button_rect(_rect: ButtonRect) {}

    pub fn maximize_button_rect() -> ButtonRect {
        ButtonRect::default()
    }
}
