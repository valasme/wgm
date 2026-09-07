//! The Settings migration chain.
//!
//! One function per version step, applied in order. Three rules, all enforced by the
//! generic module in `document/mod.rs`:
//!
//! - **Pure and total.** No I/O, no clock, no environment. Given the same JSON in, a
//!   migration always produces the same JSON out.
//! - **Never panic.** A panic is caught and treated exactly as corruption, so a
//!   migration that panics costs the user their file. Return `Err` instead.
//! - **Every step has a fixture.** `tests/fixtures/settings/` holds the input and the
//!   expected output for each one.

use serde_json::{Map, Value};

/// Apply the step that takes a document *from* `version` to `version + 1`.
pub fn step(value: &mut Map<String, Value>, version: u32) -> Result<(), String> {
    match version {
        0 => v0_to_v1(value),
        1 => v1_to_v2(value),
        other => Err(format!("no migration is defined from version {other}")),
    }
}

/// Version 0 is a Settings File with no `schemaVersion` field at all: one written by
/// hand, or by a build from before the field existed.
///
/// There is nothing to rename — the shape has not changed — so this step exists to
/// stamp the version and to let the document module rewrite the file in the current
/// format. It is a real step rather than a no-op special case, because "a file with
/// no version is version 0" has to be true somewhere, and encoding it here is what
/// keeps the loader free of exceptions.
fn v0_to_v1(value: &mut Map<String, Value>) -> Result<(), String> {
    // A pre-versioning file that is not an object of sections is not a Settings File
    // at all, and should be treated as corrupt rather than silently emptied.
    for section in ["appearance", "general", "advanced", "privacy"] {
        match value.get(section) {
            None | Some(Value::Object(_)) => {}
            Some(other) => {
                return Err(format!("`{section}` is {other}, not an object"));
            }
        }
    }

    value.insert("schemaVersion".to_owned(), Value::from(1));

    Ok(())
}

/// Version 2 drops `general.launchOnStartup` and `general.startMinimized`.
///
/// Both were removed rather than defaulted: wgm no longer writes a registry run entry
/// or opens minimised, so the fields describe behaviour that does not exist. Dropping
/// them here rather than leaving them in the file is what keeps them out of the
/// document module's unknown-field report on every subsequent load.
fn v1_to_v2(value: &mut Map<String, Value>) -> Result<(), String> {
    if let Some(Value::Object(general)) = value.get_mut("general") {
        general.remove("launchOnStartup");
        general.remove("startMinimized");
    }

    value.insert("schemaVersion".to_owned(), Value::from(2));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn object(json: &str) -> Map<String, Value> {
        serde_json::from_str(json).expect("the fixture must be a JSON object")
    }

    #[test]
    fn v0_gains_a_schema_version_and_keeps_its_values() {
        let mut value = object(r#"{"appearance":{"accent":"rose"}}"#);

        step(&mut value, 0).expect("the v0 step must succeed");

        assert_eq!(value["schemaVersion"], Value::from(1));
        assert_eq!(value["appearance"]["accent"], Value::from("rose"));
    }

    #[test]
    fn a_section_of_the_wrong_shape_is_reported_rather_than_dropped() {
        let mut value = object(r#"{"appearance": "dark"}"#);

        let error = step(&mut value, 0).expect_err("a string section is not migratable");

        assert!(
            error.contains("appearance"),
            "the error must name the section: {error}"
        );
    }

    #[test]
    fn an_unknown_step_is_an_error_rather_than_a_silent_success() {
        let mut value = object("{}");
        assert!(step(&mut value, 99).is_err());
    }

    #[test]
    fn v1_drops_the_two_startup_settings_and_keeps_the_rest() {
        let mut value = object(
            r#"{"schemaVersion":1,"general":{"launchOnStartup":true,"startMinimized":true,"restoreWindowPosition":false}}"#,
        );

        step(&mut value, 1).expect("the v1 step must succeed");

        assert_eq!(value["schemaVersion"], Value::from(2));
        let general = value["general"].as_object().expect("general is an object");
        assert!(!general.contains_key("launchOnStartup"));
        assert!(!general.contains_key("startMinimized"));
        assert_eq!(general["restoreWindowPosition"], Value::from(false));
    }

    #[test]
    fn migrations_are_deterministic() {
        let mut first = object(r#"{"general":{"startMinimized":true}}"#);
        let mut second = first.clone();

        step(&mut first, 0).expect("step");
        step(&mut second, 0).expect("step");

        assert_eq!(first, second);
    }
}
