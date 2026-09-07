//! The Workspace State migration chain. Same three rules as the Settings chain: pure,
//! total, never panicking, and every step has a fixture.

use serde_json::{Map, Value};

pub fn step(value: &mut Map<String, Value>, version: u32) -> Result<(), String> {
    match version {
        0 => v0_to_v1(value),
        other => Err(format!("no migration is defined from version {other}")),
    }
}

/// Version 0 is a Workspace State file with no `schemaVersion` — written by hand, or
/// by a build from before the field existed.
fn v0_to_v1(value: &mut Map<String, Value>) -> Result<(), String> {
    for section in ["window", "sidebar", "onboarding"] {
        match value.get(section) {
            None | Some(Value::Object(_)) => {}
            Some(other) => return Err(format!("`{section}` is {other}, not an object")),
        }
    }

    // A pre-versioning file may still carry the boolean this field replaced. Onboarding
    // progress is a version string so a future release can replay one step; reading an
    // old `true` as "completed at some unknown version" is the honest translation.
    if let Some(Value::Object(onboarding)) = value.get_mut("onboarding") {
        if let Some(Value::Bool(completed)) = onboarding.remove("completed") {
            onboarding.insert(
                "completedVersion".to_owned(),
                if completed {
                    Value::from("0.0.0")
                } else {
                    Value::Null
                },
            );
        }
    }

    value.insert("schemaVersion".to_owned(), Value::from(1));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn object(json: &str) -> Map<String, Value> {
        serde_json::from_str(json).expect("the fixture must be a JSON object")
    }

    #[test]
    fn v0_gains_a_schema_version() {
        let mut value = object(r#"{"sidebar":{"width":300}}"#);

        step(&mut value, 0).expect("the v0 step must succeed");

        assert_eq!(value["schemaVersion"], Value::from(1));
        assert_eq!(value["sidebar"]["width"], Value::from(300));
    }

    #[test]
    fn a_boolean_onboarding_flag_becomes_a_version() {
        let mut value = object(r#"{"onboarding":{"completed":true}}"#);

        step(&mut value, 0).expect("step");

        assert_eq!(
            value["onboarding"]["completedVersion"],
            Value::from("0.0.0")
        );
        assert!(value["onboarding"].get("completed").is_none());
    }

    #[test]
    fn an_incomplete_boolean_becomes_null_rather_than_a_version() {
        let mut value = object(r#"{"onboarding":{"completed":false}}"#);

        step(&mut value, 0).expect("step");

        assert_eq!(value["onboarding"]["completedVersion"], Value::Null);
    }

    #[test]
    fn an_unknown_step_is_an_error() {
        assert!(step(&mut object("{}"), 42).is_err());
    }
}
