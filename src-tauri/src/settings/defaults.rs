//! Defaults, and the clamping rules that keep a hand-edited file usable.
//!
//! `CONTEXT.md`: Defaults are the compiled-in baseline Settings — the fallback when a
//! field is missing or a file is unreadable, and **never itself persisted as a
//! separate document**. There is no `defaults.json`; `Settings::default()` is it.

use serde_json::{Map, Value};

use crate::document::{clamp_enum, clamp_u32};
use crate::settings::schema::{
    Accent, ColorSchemePreference, Density, LogLevel, ReduceMotion, Settings, LOG_RETENTION_RANGE,
};

/// The compiled-in baseline.
pub fn defaults() -> Settings {
    Settings::default()
}

/// Pull every out-of-range value back inside its range, returning the dotted names of
/// the fields that changed.
///
/// Clamping rather than rejecting is the rule from `docs/failure-modes.md` §2: one bad
/// number must not cost the user every other setting in the file. An unrecognised enum
/// is treated the same way — it resets to the default for that one field, rather than
/// failing deserialisation and taking the whole document with it.
pub fn clamp(value: &mut Map<String, Value>) -> Vec<String> {
    let mut changed = Vec::new();

    if let Some(Value::Object(appearance)) = value.get_mut("appearance") {
        let fallbacks: [(&str, &[&str], &str); 4] = [
            ("colorScheme", ColorSchemePreference::ALLOWED, "system"),
            ("accent", Accent::ALLOWED, "blue"),
            ("density", Density::ALLOWED, "comfortable"),
            ("reduceMotion", ReduceMotion::ALLOWED, "system"),
        ];

        for (key, allowed, fallback) in fallbacks {
            if clamp_enum(appearance, key, allowed, fallback) {
                changed.push(format!("appearance.{key}"));
            }
        }
    }

    if let Some(Value::Object(advanced)) = value.get_mut("advanced") {
        if clamp_enum(advanced, "logLevel", LogLevel::ALLOWED, "info") {
            changed.push("advanced.logLevel".to_owned());
        }
        if clamp_u32(advanced, "logRetentionDays", LOG_RETENTION_RANGE) {
            changed.push("advanced.logRetentionDays".to_owned());
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
    fn an_over_long_retention_is_clamped_to_the_maximum() {
        let mut value = object(r#"{"advanced":{"logRetentionDays":100000}}"#);

        let changed = clamp(&mut value);

        assert_eq!(changed, vec!["advanced.logRetentionDays".to_owned()]);
        assert_eq!(value["advanced"]["logRetentionDays"], Value::from(90));
    }

    #[test]
    fn a_zero_retention_is_clamped_up_rather_than_disabling_logging() {
        let mut value = object(r#"{"advanced":{"logRetentionDays":0}}"#);

        clamp(&mut value);

        assert_eq!(value["advanced"]["logRetentionDays"], Value::from(1));
    }

    #[test]
    fn an_invented_accent_resets_only_that_field() {
        let mut value = object(r#"{"appearance":{"accent":"chartreuse","colorScheme":"darker"}}"#);

        let changed = clamp(&mut value);

        assert_eq!(changed, vec!["appearance.accent".to_owned()]);
        assert_eq!(value["appearance"]["accent"], Value::from("blue"));
        assert_eq!(
            value["appearance"]["colorScheme"],
            Value::from("darker"),
            "a neighbouring valid field must survive"
        );
    }

    #[test]
    fn a_document_already_in_range_is_left_alone() {
        let mut value = serde_json::to_value(defaults())
            .expect("serialise")
            .as_object()
            .cloned()
            .expect("Settings is an object");

        assert!(clamp(&mut value).is_empty());
    }
}
