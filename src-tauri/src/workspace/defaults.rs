//! Workspace State defaults and clamping.
//!
//! Geometry needs clamping more than Settings does: a window position is written by
//! one monitor layout and read by another. A saved position on a monitor that has
//! since been unplugged puts the window somewhere the user cannot reach it, and a
//! saved size below the minimum makes the chrome overlap itself.

use serde_json::{Map, Value};

use crate::document::clamp_u32;
use crate::workspace::schema::{
    WorkspaceState, SIDEBAR_WIDTH_RANGE, WINDOW_MIN_HEIGHT, WINDOW_MIN_WIDTH,
};

pub fn defaults() -> WorkspaceState {
    WorkspaceState::default()
}

pub fn clamp(value: &mut Map<String, Value>) -> Vec<String> {
    let mut changed = Vec::new();

    if let Some(Value::Object(sidebar)) = value.get_mut("sidebar") {
        if clamp_u32(sidebar, "width", SIDEBAR_WIDTH_RANGE) {
            changed.push("sidebar.width".to_owned());
        }
    }

    if let Some(Value::Object(window)) = value.get_mut("window") {
        // Upper bounds are generous rather than screen-derived: the monitor layout at
        // load time is not necessarily the one that will be in effect when the window
        // is shown, and Windows clamps a too-large window to the work area anyway.
        if clamp_u32(window, "width", WINDOW_MIN_WIDTH..=32_000) {
            changed.push("window.width".to_owned());
        }
        if clamp_u32(window, "height", WINDOW_MIN_HEIGHT..=32_000) {
            changed.push("window.height".to_owned());
        }
    }

    changed
}

#[cfg(test)]
mod tests {
    use super::*;

    fn object(json: &str) -> Map<String, Value> {
        serde_json::from_str(json).expect("the fixture must be a JSON object")
    }

    #[test]
    fn a_sidebar_dragged_past_its_bounds_is_pulled_back() {
        let mut value = object(r#"{"sidebar":{"width":4000}}"#);

        assert_eq!(clamp(&mut value), vec!["sidebar.width".to_owned()]);
        assert_eq!(value["sidebar"]["width"], Value::from(400));
    }

    #[test]
    fn a_window_saved_smaller_than_the_minimum_is_restored_to_the_minimum() {
        let mut value = object(r#"{"window":{"width":10,"height":10}}"#);

        let changed = clamp(&mut value);

        assert_eq!(
            changed,
            vec!["window.width".to_owned(), "window.height".to_owned()]
        );
        assert_eq!(value["window"]["width"], Value::from(WINDOW_MIN_WIDTH));
        assert_eq!(value["window"]["height"], Value::from(WINDOW_MIN_HEIGHT));
    }

    #[test]
    fn defaults_need_no_clamping() {
        let mut value = serde_json::to_value(defaults())
            .expect("serialise")
            .as_object()
            .cloned()
            .expect("WorkspaceState is an object");

        assert!(clamp(&mut value).is_empty());
    }
}
