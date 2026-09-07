//! Window placement decisions, as pure functions.

use crate::workspace::schema::WindowGeometry;

/// One monitor's work area, in the same virtual-desktop coordinates a window position
/// is saved in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Monitor {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// What to do to the window before it is shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Placement {
    pub width: u32,
    pub height: u32,
    /// `None` leaves the window where the OS centred it.
    pub position: Option<(i32, i32)>,
    pub maximized: bool,
}

/// Decide how to place the window at startup.
///
/// `None` means "change nothing" — the window keeps the size and position
/// `tauri.conf.json` and the OS gave it.
pub fn restore_plan(
    saved: WindowGeometry,
    restore_enabled: bool,
    monitors: &[Monitor],
) -> Option<Placement> {
    if !restore_enabled {
        return None;
    }

    let position = match (saved.x, saved.y) {
        (Some(x), Some(y)) if on_a_monitor(x, y, saved.width, saved.height, monitors) => {
            Some((x, y))
        }
        _ => None,
    };

    Some(Placement {
        width: saved.width,
        height: saved.height,
        position,
        maximized: saved.maximized,
    })
}

/// What the window says about itself after a move or a resize.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowReport {
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
    pub maximized: bool,
    pub minimized: bool,
}

/// Decide what to persist after the window moved or was resized.
///
/// `None` means "write nothing".
pub fn capture(previous: WindowGeometry, report: WindowReport) -> Option<WindowGeometry> {
    // A minimized window is parked off-screen by Windows and reports nothing true
    // about where the user left it.
    if report.minimized {
        return None;
    }

    // A maximized window reports the work area, not the size the user chose. Only the
    // flag is news; the bounds underneath it are the ones to restore down to.
    let candidate = if report.maximized {
        WindowGeometry {
            maximized: true,
            ..previous
        }
    } else {
        WindowGeometry {
            width: report.width,
            height: report.height,
            x: Some(report.x),
            y: Some(report.y),
            maximized: false,
        }
    };

    (candidate != previous).then_some(candidate)
}

/// The window has to land on a monitor by at least this much to count as reachable:
/// enough Title Bar to grab and drag the rest back into view. The height is the Title
/// Bar's own 36px.
const MIN_VISIBLE_WIDTH: i32 = 120;
const MIN_VISIBLE_HEIGHT: i32 = 36;

/// Does a window at this rect land on any of these monitors?
///
/// The saved position is written by one monitor layout and read by another. A window
/// restored onto a monitor that has since been unplugged is invisible and unreachable,
/// and the user's only recourse is deleting a file they do not know about.
fn on_a_monitor(x: i32, y: i32, width: u32, height: u32, monitors: &[Monitor]) -> bool {
    let right = x.saturating_add(width as i32);
    let bottom = y.saturating_add(height as i32);

    monitors.iter().any(|monitor| {
        let monitor_right = monitor.x.saturating_add(monitor.width as i32);
        let monitor_bottom = monitor.y.saturating_add(monitor.height as i32);

        let visible_width = right.min(monitor_right) - x.max(monitor.x);
        let visible_height = bottom.min(monitor_bottom) - y.max(monitor.y);

        visible_width >= MIN_VISIBLE_WIDTH && visible_height >= MIN_VISIBLE_HEIGHT
    })
}

#[cfg(test)]
mod tests {
    use crate::workspace::geometry::{capture, restore_plan, Monitor, Placement, WindowReport};
    use crate::workspace::schema::WindowGeometry;

    /// A 1000x700 window at (120, 80), the state most tests start from.
    const LEFT_AT: WindowGeometry = WindowGeometry {
        width: 1000,
        height: 700,
        x: Some(120),
        y: Some(80),
        maximized: false,
    };

    /// A single 1920x1080 monitor at the origin.
    const PRIMARY: Monitor = Monitor {
        x: 0,
        y: 0,
        width: 1920,
        height: 1080,
    };

    #[test]
    fn the_setting_being_off_leaves_the_window_where_the_os_put_it() {
        let saved = WindowGeometry {
            width: 1000,
            height: 700,
            x: Some(120),
            y: Some(80),
            maximized: false,
        };

        assert_eq!(restore_plan(saved, false, &[]), None);
    }

    #[test]
    fn a_window_reopens_at_the_size_and_place_it_was_left() {
        let saved = WindowGeometry {
            width: 1000,
            height: 700,
            x: Some(120),
            y: Some(80),
            maximized: false,
        };

        assert_eq!(
            restore_plan(saved, true, &[PRIMARY]),
            Some(Placement {
                width: 1000,
                height: 700,
                position: Some((120, 80)),
                maximized: false,
            })
        );
    }

    #[test]
    fn a_position_on_a_monitor_that_is_gone_is_dropped_but_the_size_is_kept() {
        // Left on a second monitor to the right of the primary one, reopened after it
        // was unplugged.
        let saved = WindowGeometry {
            width: 1000,
            height: 700,
            x: Some(2400),
            y: Some(300),
            maximized: false,
        };

        assert_eq!(
            restore_plan(saved, true, &[PRIMARY]),
            Some(Placement {
                width: 1000,
                height: 700,
                position: None,
                maximized: false,
            })
        );
    }

    #[test]
    fn a_window_with_only_a_sliver_on_screen_is_treated_as_unreachable() {
        // 70px across and 20px down of the window overlap the monitor — not enough
        // Title Bar to grab, so this is no more usable than being fully off-screen.
        let saved = WindowGeometry {
            width: 1000,
            height: 700,
            x: Some(1850),
            y: Some(1060),
            maximized: false,
        };

        assert_eq!(
            restore_plan(saved, true, &[PRIMARY]).expect("the size is still restored"),
            Placement {
                width: 1000,
                height: 700,
                position: None,
                maximized: false,
            }
        );
    }

    #[test]
    fn moving_and_resizing_the_window_records_the_new_bounds() {
        let report = WindowReport {
            width: 1400,
            height: 900,
            x: 40,
            y: 20,
            maximized: false,
            minimized: false,
        };

        assert_eq!(
            capture(LEFT_AT, report),
            Some(WindowGeometry {
                width: 1400,
                height: 900,
                x: Some(40),
                y: Some(20),
                maximized: false,
            })
        );
    }

    #[test]
    fn maximizing_records_the_flag_and_keeps_the_bounds_to_restore_down_to() {
        // A maximized window reports the whole work area, plus the invisible border
        // Windows hangs off each edge. Writing that as the geometry would lose the
        // size the user actually chose, so restoring down after a restart would land
        // on a full-screen-sized "restored" window.
        let report = WindowReport {
            width: 1936,
            height: 1096,
            x: -8,
            y: -8,
            maximized: true,
            minimized: false,
        };

        assert_eq!(
            capture(LEFT_AT, report),
            Some(WindowGeometry {
                maximized: true,
                ..LEFT_AT
            })
        );
    }

    #[test]
    fn minimizing_writes_nothing() {
        // Windows parks a minimized window at (-32000, -32000). Recording that would
        // throw the real position away — and *Start minimised* minimises during boot,
        // so it would happen on every launch for anyone using that setting.
        let report = WindowReport {
            width: 160,
            height: 28,
            x: -32_000,
            y: -32_000,
            maximized: false,
            minimized: true,
        };

        assert_eq!(capture(LEFT_AT, report), None);
    }

    #[test]
    fn a_window_that_has_not_actually_moved_writes_nothing() {
        // Restoring the geometry at startup makes the window emit its own move and
        // resize events. Writing the file back out on each of those would mean a disk
        // write on every launch that changed nothing.
        let report = WindowReport {
            width: 1000,
            height: 700,
            x: 120,
            y: 80,
            maximized: false,
            minimized: false,
        };

        assert_eq!(capture(LEFT_AT, report), None);
    }
}
