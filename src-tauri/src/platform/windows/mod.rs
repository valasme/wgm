//! Windows-specific window behaviour.
//!
//! wgm draws its own Title Bar, which means `decorations: false`, which means Windows
//! stops doing three things it normally does for free. Each is restored here:
//!
//! | Lost | Restored by |
//! | --- | --- |
//! | Snap Layouts on maximise-button hover | `WM_NCHITTEST` returning `HTMAXBUTTON` |
//! | A maximised window that respects the taskbar | `WM_GETMINMAXINFO` clamping to the work area |
//! | `Alt+Space` opening the system menu | [`system_menu`] |
//!
//! All of it is behind `#[cfg(windows)]` and one kill switch, [`ENABLE_NATIVE_HIT_TEST`].

pub mod system_menu;
pub mod window_proc;

use std::sync::{Mutex, OnceLock};

use serde::{Deserialize, Serialize};
use specta::Type;

/// Turn the native hit-testing off in one place.
///
/// It is a window subclass reaching into the OS message loop; if it ever misbehaves on
/// a Windows build we do not have, flipping this to `false` restores plain behaviour
/// without unpicking anything. The Title Bar keeps working — only Snap Layouts on
/// hover and the work-area clamp are lost.
pub const ENABLE_NATIVE_HIT_TEST: bool = true;

/// Where the maximise button is, in physical pixels relative to the window.
///
/// The frontend owns the layout, so the frontend reports this: Rust cannot know where
/// a React component ended up. `WM_NCHITTEST` reads it to decide when to answer
/// `HTMAXBUTTON`, which is what makes Snap Layouts appear on hover.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ButtonRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl ButtonRect {
    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }

    pub fn is_empty(&self) -> bool {
        self.width <= 0 || self.height <= 0
    }
}

fn rect_cell() -> &'static Mutex<ButtonRect> {
    static RECT: OnceLock<Mutex<ButtonRect>> = OnceLock::new();
    RECT.get_or_init(|| Mutex::new(ButtonRect::default()))
}

/// Called from the frontend whenever the Window Controls are laid out or resized.
pub fn set_maximize_button_rect(rect: ButtonRect) {
    if let Ok(mut cell) = rect_cell().lock() {
        *cell = rect;
    }
}

pub fn maximize_button_rect() -> ButtonRect {
    rect_cell().lock().map(|rect| *rect).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_point_inside_the_button_is_inside() {
        let rect = ButtonRect {
            x: 100,
            y: 0,
            width: 46,
            height: 36,
        };

        assert!(rect.contains(100, 0));
        assert!(rect.contains(145, 35));
        assert!(!rect.contains(146, 35), "the right edge is exclusive");
        assert!(!rect.contains(99, 10));
        assert!(!rect.contains(120, 36), "the bottom edge is exclusive");
    }

    #[test]
    fn an_unset_rect_is_empty_and_never_claims_a_hit() {
        let rect = ButtonRect::default();

        assert!(rect.is_empty());
        assert!(!rect.contains(0, 0));
    }

    #[test]
    fn the_rect_round_trips_through_the_global_cell() {
        let rect = ButtonRect {
            x: 1,
            y: 2,
            width: 3,
            height: 4,
        };

        set_maximize_button_rect(rect);

        assert_eq!(maximize_button_rect(), rect);

        set_maximize_button_rect(ButtonRect::default());
    }
}
