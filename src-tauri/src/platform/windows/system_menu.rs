//! `Alt+Space` — the real Windows system menu.
//!
//! Undecorated windows lose it by default, and it is how screen reader users have
//! always reached Move, Size, Minimise, Maximise and Close. A custom Title Bar that
//! takes it away has traded an accessibility affordance for a visual one.
//!
//! `Alt+F4` and `Win+Arrow` are handled by Windows itself and continue to work
//! untouched.

#![cfg(windows)]

use windows::Win32::Foundation::{POINT, RECT};
use windows::Win32::UI::WindowsAndMessaging::{
    GetSystemMenu, GetWindowRect, PostMessageW, TrackPopupMenu, HMENU, TPM_LEFTALIGN,
    TPM_RETURNCMD, TPM_TOPALIGN, WM_SYSCOMMAND,
};

/// Open the system menu at the window's top-left corner, where Windows itself puts it.
///
/// Called from a command the frontend invokes on `Alt+Space`, rather than from a
/// global shortcut: a global hook would swallow `Alt+Space` for every application.
pub fn open(window: &tauri::WebviewWindow) -> Result<(), String> {
    let handle = window
        .hwnd()
        .map_err(|error| format!("no window handle: {error}"))?;

    // SAFETY: `handle` is a live window owned by this process. `GetSystemMenu` with
    // `false` returns the window's menu without resetting it, and the handle it
    // returns is owned by the window rather than by us.
    unsafe {
        let menu: HMENU = GetSystemMenu(handle, false);
        if menu.is_invalid() {
            return Err("the window has no system menu".to_owned());
        }

        let mut bounds = RECT::default();
        GetWindowRect(handle, &mut bounds)
            .map_err(|error| format!("GetWindowRect failed: {error}"))?;

        let origin = POINT {
            x: bounds.left,
            y: bounds.top,
        };

        let command = TrackPopupMenu(
            menu,
            TPM_LEFTALIGN | TPM_TOPALIGN | TPM_RETURNCMD,
            origin.x,
            origin.y,
            None,
            handle,
            None,
        );

        // TPM_RETURNCMD means the menu returns the chosen command instead of posting
        // it, so it has to be posted here. Zero means the user dismissed the menu.
        if command.0 != 0 {
            PostMessageW(
                Some(handle),
                WM_SYSCOMMAND,
                windows::Win32::Foundation::WPARAM(command.0 as usize),
                windows::Win32::Foundation::LPARAM(0),
            )
            .map_err(|error| format!("PostMessageW failed: {error}"))?;
        }
    }

    Ok(())
}
