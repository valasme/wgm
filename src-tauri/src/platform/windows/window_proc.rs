//! One window subclass, handling both messages an undecorated window loses.
//!
//! **`WM_NCHITTEST`** — Windows 11 shows the Snap Layouts flyout when the pointer
//! rests over a button that reports `HTMAXBUTTON`. An undecorated window has no
//! non-client area and so never reports it, and the feature silently disappears. The
//! frontend tells us where its maximise button is; this answers accordingly.
//!
//! **`WM_GETMINMAXINFO`** — without it a maximised undecorated window sizes itself to
//! the whole *monitor* rather than the *work area*, and covers the taskbar. This is
//! the single most visible defect a custom title bar introduces.
//!
//! Subclassing is done with `SetWindowLongPtrW(GWLP_WNDPROC)` rather than comctl32's
//! `SetWindowSubclass`. The comctl32 helpers are exported only by version 6, which a
//! process gets from WinSxS *if it has an application manifest asking for it* — so a
//! statically imported `SetWindowSubclass` makes any binary without that manifest,
//! `cargo test`'s harness included, fail to load at all with
//! `STATUS_ENTRYPOINT_NOT_FOUND`. `SetWindowLongPtrW` is user32 and always present.

#![cfg(windows)]

use std::sync::atomic::{AtomicIsize, Ordering};

use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallWindowProcW, GetWindowRect, SetWindowLongPtrW, GWLP_WNDPROC, MINMAXINFO, WM_GETMINMAXINFO,
    WM_NCHITTEST, WM_NCLBUTTONDOWN, WM_NCLBUTTONUP, WNDPROC,
};

use super::{maximize_button_rect, ENABLE_NATIVE_HIT_TEST};

/// `HTMAXBUTTON`, from `winuser.h`. Restated here next to the code that depends on
/// what it means.
const HTMAXBUTTON: isize = 9;
/// `HTCLIENT` — "the pointer is over ordinary content", the answer everywhere that is
/// not the maximise button.
const HTCLIENT: isize = 1;

/// The window proc we replaced, to chain to. wgm has exactly one window, so one slot
/// is enough; a second window would need a map keyed by `HWND`.
static PREVIOUS_PROC: AtomicIsize = AtomicIsize::new(0);

/// Install the subclass on a Tauri window.
///
/// Returns without doing anything when [`ENABLE_NATIVE_HIT_TEST`] is off, so the kill
/// switch is a single edit rather than an unpicking exercise.
pub fn install(window: &tauri::WebviewWindow) -> Result<(), String> {
    if !ENABLE_NATIVE_HIT_TEST {
        log::info!(target: "wgm::platform", "native hit testing is disabled by kill switch");
        return Ok(());
    }

    let handle = window
        .hwnd()
        .map_err(|error| format!("no window handle: {error}"))?;

    if PREVIOUS_PROC.load(Ordering::Acquire) != 0 {
        // Installing twice would chain the proc to itself and hang the message loop.
        return Ok(());
    }

    // SAFETY: `handle` is a live top-level window owned by this process, and
    // `window_proc` has the signature `WNDPROC` requires.
    let previous =
        unsafe { SetWindowLongPtrW(handle, GWLP_WNDPROC, window_proc as *const () as isize) };

    if previous == 0 {
        return Err("SetWindowLongPtrW(GWLP_WNDPROC) returned 0".to_owned());
    }

    PREVIOUS_PROC.store(previous, Ordering::Release);
    log::info!(target: "wgm::platform", "window proc installed");

    Ok(())
}

/// Chain to the proc we replaced.
///
/// # Safety
/// Only called from inside [`window_proc`], with arguments Windows itself supplied.
unsafe fn chain(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let previous = PREVIOUS_PROC.load(Ordering::Acquire);

    // Zero would mean install() never ran, which cannot happen from inside our own
    // proc — but defaulting to DefWindowProcW rather than transmuting a null pointer
    // is the difference between a wrong pixel and an access violation.
    if previous == 0 {
        return windows::Win32::UI::WindowsAndMessaging::DefWindowProcW(
            window, message, wparam, lparam,
        );
    }

    let previous: WNDPROC = Some(std::mem::transmute::<
        isize,
        unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT,
    >(previous));

    CallWindowProcW(previous, window, message, wparam, lparam)
}

unsafe extern "system" fn window_proc(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_NCHITTEST => hit_test(window, wparam, lparam),
        WM_GETMINMAXINFO => {
            clamp_to_work_area(window, lparam);
            LRESULT(0)
        }
        // Windows sends these to the *non-client* area we have just claimed. Swallow
        // them so a click on the maximise button reaches our React handler rather than
        // also starting a native maximise, which would toggle twice.
        WM_NCLBUTTONDOWN | WM_NCLBUTTONUP
            if wparam.0 as isize == HTMAXBUTTON && !maximize_button_rect().is_empty() =>
        {
            LRESULT(0)
        }
        _ => chain(window, message, wparam, lparam),
    }
}

/// Answer `HTMAXBUTTON` while the pointer is over the maximise button.
unsafe fn hit_test(window: HWND, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let rect = maximize_button_rect();

    if rect.is_empty() {
        // The frontend has not reported a rect yet — during boot, or with the Title
        // Bar not yet mounted. Claiming a hit here would make the whole window behave
        // like a maximise button.
        return chain(window, WM_NCHITTEST, wparam, lparam);
    }

    // lparam carries screen coordinates packed as two signed 16-bit values.
    let screen_x = (lparam.0 & 0xFFFF) as i16 as i32;
    let screen_y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;

    let mut window_rect = RECT::default();
    if GetWindowRect(window, &mut window_rect).is_err() {
        return chain(window, WM_NCHITTEST, wparam, lparam);
    }

    if rect.contains(screen_x - window_rect.left, screen_y - window_rect.top) {
        LRESULT(HTMAXBUTTON)
    } else {
        // Not the chained proc: an undecorated window's default answer over the title
        // bar can be HTCAPTION, and returning that here would fight
        // `data-tauri-drag-region`.
        LRESULT(HTCLIENT)
    }
}

/// Clamp a maximised window to the monitor's **work area**, not its full bounds.
unsafe fn clamp_to_work_area(window: HWND, lparam: LPARAM) {
    let monitor = MonitorFromWindow(window, MONITOR_DEFAULTTONEAREST);

    let mut info = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };

    if !GetMonitorInfoW(monitor, &mut info).as_bool() {
        return;
    }

    let min_max = lparam.0 as *mut MINMAXINFO;
    if min_max.is_null() {
        return;
    }

    let work = info.rcWork;
    let bounds = info.rcMonitor;

    // Positions are relative to the monitor's own origin, which is not the desktop
    // origin on a multi-monitor setup with a display to the left of the primary.
    (*min_max).ptMaxPosition.x = work.left - bounds.left;
    (*min_max).ptMaxPosition.y = work.top - bounds.top;
    (*min_max).ptMaxSize.x = work.right - work.left;
    (*min_max).ptMaxSize.y = work.bottom - work.top;
}
