//! The Workspace State schema.
//!
//! Everything true only of *this* installation on *this* machine: window geometry,
//! Sidebar width, Rail state, onboarding progress. **Never exported, never imported.**
//!
//! This is also why `tauri-plugin-window-state` is not used: it saves to the app
//! config directory with no path override, which would put window geometry in
//! `%APPDATA%` while Settings sat in `./data/` under portable mode.

use serde::{Deserialize, Serialize};
use specta::Type;

pub const CURRENT_VERSION: u32 = 1;

/// Sidebar width bounds. The resizer's `aria-valuemin` and `aria-valuemax` are these
/// numbers, and so is the clamp on load.
pub const SIDEBAR_WIDTH_RANGE: std::ops::RangeInclusive<u64> = 180..=400;
pub const SIDEBAR_WIDTH_DEFAULT: u32 = 220;

/// The minimum window size, mirrored from `tauri.conf.json`. Geometry restored from a
/// file is clamped to it, because a monitor that has since been unplugged can leave a
/// window at a size or position that is unreachable.
pub const WINDOW_MIN_WIDTH: u64 = 680;
pub const WINDOW_MIN_HEIGHT: u64 = 480;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase", default)]
pub struct WindowGeometry {
    pub width: u32,
    pub height: u32,
    /// Absent until the window has been moved, so a first run is centred by the OS.
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub maximized: bool,
}

impl Default for WindowGeometry {
    fn default() -> Self {
        WindowGeometry {
            width: 1200,
            height: 760,
            x: None,
            y: None,
            maximized: false,
        }
    }
}

/// The Sidebar and its collapsed form, the Rail. One component in two states, not two
/// components — so this is one field, not two.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase", default)]
pub struct SidebarState {
    pub width: u32,
    /// True when the user collapsed it to the Rail. Below a 900px window width the
    /// Rail is forced regardless, and that is *not* recorded here — a narrow window
    /// must not permanently collapse the Sidebar.
    pub collapsed: bool,
}

impl Default for SidebarState {
    fn default() -> Self {
        SidebarState {
            width: SIDEBAR_WIDTH_DEFAULT,
            collapsed: false,
        }
    }
}

/// Onboarding progress.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Onboarding {
    /// The wgm version whose setup the user completed — **not a boolean**, so a
    /// future major release can replay a single "what's new" step without replaying
    /// the whole thing. `None` means setup has never been finished.
    pub completed_version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase", default)]
pub struct WorkspaceState {
    pub schema_version: u32,
    pub window: WindowGeometry,
    pub sidebar: SidebarState,
    pub onboarding: Onboarding,
}

impl Default for WorkspaceState {
    fn default() -> Self {
        WorkspaceState {
            schema_version: CURRENT_VERSION,
            window: WindowGeometry::default(),
            sidebar: SidebarState::default(),
            onboarding: Onboarding::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_the_documented_chrome_metrics() {
        let state = WorkspaceState::default();

        assert_eq!(state.sidebar.width, 220);
        assert!(!state.sidebar.collapsed);
        assert_eq!(state.window.width, 1200);
        assert_eq!(state.window.height, 760);
        assert_eq!(state.onboarding.completed_version, None);
    }

    #[test]
    fn onboarding_progress_is_a_version_not_a_boolean() {
        let raw = serde_json::to_string(&Onboarding {
            completed_version: Some("0.1.0".into()),
        })
        .expect("serialise");

        assert_eq!(raw, r#"{"completedVersion":"0.1.0"}"#);
    }

    #[test]
    fn an_empty_object_round_trips_to_defaults() {
        let state: WorkspaceState = serde_json::from_str("{}").expect("every field has a default");
        assert_eq!(state, WorkspaceState::default());
    }
}
