//! The Tauri side of window geometry: read the window, write the document.
//!
//! Every decision this makes lives in [`crate::workspace::geometry`] as a pure
//! function. What is left here is the part that needs a real window — asking it where
//! it is, moving it, and coalescing a drag into one file write.

use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::thread;
use std::time::Duration;

use tauri::{AppHandle, LogicalPosition, LogicalSize, Manager, WebviewWindow, WindowEvent};

use crate::state::AppState;
use crate::workspace::geometry::{capture, restore_plan, Monitor, WindowReport};
use crate::workspace::schema::WindowGeometry;

/// How long the window has to sit still before its geometry is written.
///
/// `Moved` fires on every frame of a drag. Without this, one trip across the desktop
/// would be a few hundred atomic file writes through a single global write lock — the
/// same reason the Sidebar resizer debounces.
const WRITE_DEBOUNCE: Duration = Duration::from_millis(500);

/// Put the window back where it was left, before it is shown.
///
/// Geometry is held in logical pixels, the same units as `tauri.conf.json` and as
/// `WINDOW_MIN_WIDTH`, so a first run — which "restores" the compiled-in defaults —
/// is a no-op at any display scale.
pub fn restore(window: &WebviewWindow, saved: WindowGeometry, restore_enabled: bool) {
    let Some(plan) = restore_plan(saved, restore_enabled, &monitors(window)) else {
        return;
    };

    // Size first: a window positioned and then resized can end up straddling an edge
    // it was meant to sit inside.
    let _ = window.set_size(LogicalSize::new(plan.width, plan.height));

    if let Some((x, y)) = plan.position {
        let _ = window.set_position(LogicalPosition::new(x, y));
    }

    if plan.maximized {
        let _ = window.maximize();
    }
}

/// Record where the user leaves the window.
///
/// Recording continues even when *Restore window position* is off: the setting
/// governs whether geometry is read at startup, not whether it is kept, so switching
/// it back on restores where the window is now rather than where it was months ago.
pub fn persist_on_change(window: &WebviewWindow) {
    let writer = spawn_writer(window.app_handle().clone());
    let source = window.clone();

    window.on_window_event(move |event| {
        if !matches!(event, WindowEvent::Moved(_) | WindowEvent::Resized(_)) {
            return;
        }

        // `Resized` is also how Tauri reports a maximize, a restore-down and a
        // minimize, so this one arm covers every geometry change there is.
        if let Some(report) = read(&source) {
            let _ = writer.send(report);
        }
    });
}

/// The writer thread. Owns the debounce so the UI thread never blocks on a file write.
fn spawn_writer(app: AppHandle) -> Sender<WindowReport> {
    let (sender, receiver) = mpsc::channel::<WindowReport>();

    thread::spawn(move || {
        while let Ok(mut latest) = receiver.recv() {
            // Drain the rest of the drag, keeping only where it ended.
            let disconnected = loop {
                match receiver.recv_timeout(WRITE_DEBOUNCE) {
                    Ok(next) => latest = next,
                    Err(RecvTimeoutError::Timeout) => break false,
                    // The window is gone. Write what it last said before leaving,
                    // rather than losing the final move on every quit.
                    Err(RecvTimeoutError::Disconnected) => break true,
                }
            };

            write(&app, latest);

            if disconnected {
                return;
            }
        }
    });

    sender
}

fn write(app: &AppHandle, report: WindowReport) {
    let state = app.state::<AppState>();
    let previous = state.workspace.get().window;

    let Some(geometry) = capture(previous, report) else {
        return;
    };

    if let Err(error) = state
        .workspace
        .update(|workspace| workspace.window = geometry)
    {
        // Deliberately not surfaced. A window that cannot remember where it was is
        // not worth an error dialog, and the Ephemeral Mode and write-failure banners
        // already say that nothing is being saved.
        log::warn!(
            target: "wgm::workspace",
            "could not persist window geometry ({code})",
            code = error.code,
        );
    }
}

/// What the window is doing right now, in logical pixels.
///
/// Outer position with inner size is the pairing that round-trips: `set_position`
/// moves the outer frame and `set_size` sets the inner one.
fn read(window: &WebviewWindow) -> Option<WindowReport> {
    let scale = window.scale_factor().ok()?;
    let position = window.outer_position().ok()?.to_logical::<f64>(scale);
    let size = window.inner_size().ok()?.to_logical::<f64>(scale);

    Some(WindowReport {
        width: size.width as u32,
        height: size.height as u32,
        x: position.x as i32,
        y: position.y as i32,
        maximized: window.is_maximized().unwrap_or(false),
        minimized: window.is_minimized().unwrap_or(false),
    })
}

/// The monitors a restored window could land on, in logical pixels.
///
/// Each rect is converted with its own scale factor, so a mixed-DPI desktop makes this
/// an approximation. It only ever answers "is this position reachable", and the
/// approximation errs towards centring a window that could have been placed exactly —
/// visible in the wrong place, never invisible in the right one.
fn monitors(window: &WebviewWindow) -> Vec<Monitor> {
    window
        .available_monitors()
        .unwrap_or_default()
        .iter()
        .map(|monitor| {
            let scale = monitor.scale_factor();
            let position = monitor.position().to_logical::<f64>(scale);
            let size = monitor.size().to_logical::<f64>(scale);

            Monitor {
                x: position.x as i32,
                y: position.y as i32,
                width: size.width as u32,
                height: size.height as u32,
            }
        })
        .collect()
}
