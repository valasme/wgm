//! The Settings schema.
//!
//! Settings holds **everything meaningful on any machine**, and therefore everything
//! that is exportable. Anything true only of this installation on this machine —
//! window geometry, Sidebar width, Rail state, onboarding progress — belongs in
//! Workspace State instead. Keeping the two apart makes the export exclusion
//! structural rather than a list someone forgets to update.
//!
//! Every field carries `#[serde(default)]` so a file missing one is repaired from
//! Defaults rather than rejected.

use serde::{Deserialize, Serialize};
use specta::Type;

/// The schema version this build writes. Bumping it requires a migration step in
/// `migrations.rs` and a fixture in `tests/fixtures/settings/`.
pub const CURRENT_VERSION: u32 = 2;

/// Which of Light, Dark or Darker is in effect.
///
/// `System` is a *preference*, not a scheme: it resolves to Light or Dark and never
/// to Darker, because the operating system has no signal for "extra dark".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase")]
pub enum ColorSchemePreference {
    #[default]
    System,
    Light,
    Dark,
    Darker,
}

impl ColorSchemePreference {
    pub const ALLOWED: &'static [&'static str] = &["system", "light", "dark", "darker"];
}

/// The single user-selected hue driving primary buttons and interactive emphasis.
///
/// Eight fixed, contrast-verified choices rather than a picker: a freeform hue would
/// let a user choose one that fails WCAG AA and there would be no way to stop them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase")]
pub enum Accent {
    #[default]
    Blue,
    Violet,
    Cyan,
    Emerald,
    Amber,
    Orange,
    Rose,
    Neutral,
}

impl Accent {
    pub const ALLOWED: &'static [&'static str] = &[
        "blue", "violet", "cyan", "emerald", "amber", "orange", "rose", "neutral",
    ];
}

/// Density changes spacing, never type size.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase")]
pub enum Density {
    #[default]
    Comfortable,
    Compact,
}

impl Density {
    pub const ALLOWED: &'static [&'static str] = &["comfortable", "compact"];
}

/// Reduced motion is a tri-state, not a boolean, so the in-app setting can override
/// the OS **in both directions** rather than only tightening it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase")]
pub enum ReduceMotion {
    #[default]
    System,
    On,
    Off,
}

impl ReduceMotion {
    pub const ALLOWED: &'static [&'static str] = &["system", "on", "off"];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase")]
pub enum LogLevel {
    Error,
    Warn,
    #[default]
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    pub const ALLOWED: &'static [&'static str] = &["error", "warn", "info", "debug", "trace"];
}

impl From<LogLevel> for crate::logging::FileLevel {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Error => crate::logging::FileLevel::Error,
            LogLevel::Warn => crate::logging::FileLevel::Warn,
            LogLevel::Info => crate::logging::FileLevel::Info,
            LogLevel::Debug => crate::logging::FileLevel::Debug,
            LogLevel::Trace => crate::logging::FileLevel::Trace,
        }
    }
}

/// Color Scheme, Accent and density together. `CONTEXT.md` calls this Appearance and
/// deliberately not "Theme", which has meant all three at different times.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Appearance {
    pub color_scheme: ColorSchemePreference,
    pub accent: Accent,
    pub density: Density,
    pub reduce_motion: ReduceMotion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase", default)]
pub struct General {
    pub restore_window_position: bool,
    /// The one exception to instant apply: this genuinely gates Reset to defaults,
    /// which is why it is rendered at all. A control that does nothing is not shown.
    pub confirm_destructive_actions: bool,
}

impl Default for General {
    fn default() -> Self {
        General {
            restore_window_position: true,
            confirm_destructive_actions: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase", default)]
pub struct Advanced {
    pub log_level: LogLevel,
    /// Days. Clamped to `LOG_RETENTION_RANGE`; a size cap applies as well, because a
    /// Trace-level session can outgrow any day-based window in hours.
    pub log_retention_days: u32,
}

/// The range `log_retention_days` is clamped to. One day is the shortest useful
/// window; ninety is longer than any bug report has ever needed.
pub const LOG_RETENTION_RANGE: std::ops::RangeInclusive<u64> = 1..=90;

impl Default for Advanced {
    fn default() -> Self {
        Advanced {
            log_level: LogLevel::Info,
            log_retention_days: 7,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase", default)]
pub struct Privacy {
    /// Whether Settings → About offers the manual wgm Release check at all. The check
    /// itself never runs on its own — see ADR-0003.
    pub release_check_enabled: bool,
}

impl Default for Privacy {
    fn default() -> Self {
        Privacy {
            release_check_enabled: true,
        }
    }
}

/// The validated, in-memory configuration object. Distinct from the file it came from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub schema_version: u32,
    pub appearance: Appearance,
    pub general: General,
    pub advanced: Advanced,
    pub privacy: Privacy,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            schema_version: CURRENT_VERSION,
            appearance: Appearance::default(),
            general: General::default(),
            advanced: Advanced::default(),
            privacy: Privacy::default(),
        }
    }
}

/// Dotted paths removed from any Settings dump that leaves the machine.
///
/// **Declared per field, deliberately.** Today no Settings field holds a secret. The
/// moment one holds a private winget source URL with credentials in it, a global path
/// regex sails straight past it — marking fields costs nothing now and is the
/// difference between a leak and a non-event later. Adding a field here is the whole
/// procedure; `redacted_dump` does the rest.
pub const REDACTED_PATHS: &[&str] = &[];

/// What a placeholder looks like in a bundle. Chosen to be greppable and obviously
/// not a value.
pub const REDACTED_PLACEHOLDER: &str = "«redacted»";

impl Settings {
    /// The Settings dump written into a Diagnostics Bundle, with every field named in
    /// [`REDACTED_PATHS`] replaced.
    pub fn redacted_dump(&self) -> serde_json::Value {
        let mut value = serde_json::to_value(self).unwrap_or(serde_json::Value::Null);

        for path in REDACTED_PATHS {
            redact_path(&mut value, path);
        }

        value
    }
}

fn redact_path(value: &mut serde_json::Value, path: &str) {
    let Some((head, rest)) = path.split_once('.') else {
        if let Some(object) = value.as_object_mut() {
            if object.contains_key(path) {
                object.insert(
                    path.to_owned(),
                    serde_json::Value::from(REDACTED_PLACEHOLDER),
                );
            }
        }
        return;
    };

    if let Some(child) = value
        .as_object_mut()
        .and_then(|object| object.get_mut(head))
    {
        redact_path(child, rest);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_the_documented_ones() {
        let settings = Settings::default();

        assert_eq!(settings.schema_version, CURRENT_VERSION);
        assert_eq!(
            settings.appearance.color_scheme,
            ColorSchemePreference::System
        );
        assert_eq!(settings.appearance.accent, Accent::Blue);
        assert_eq!(settings.advanced.log_level, LogLevel::Info);
        assert_eq!(settings.advanced.log_retention_days, 7);
        assert!(settings.general.confirm_destructive_actions);
        assert!(settings.general.restore_window_position);
        assert!(settings.privacy.release_check_enabled);
    }

    #[test]
    fn an_empty_object_round_trips_to_defaults() {
        let settings: Settings = serde_json::from_str("{}").expect("every field has a default");
        assert_eq!(settings, Settings::default());
    }

    #[test]
    fn the_allowed_lists_match_the_enums() {
        // These lists drive the clamp path, so a variant added to the enum but not to
        // ALLOWED would be reset to the fallback on every load.
        for (allowed, rendered) in [
            (ColorSchemePreference::ALLOWED, "colorScheme"),
            (Accent::ALLOWED, "accent"),
            (Density::ALLOWED, "density"),
            (ReduceMotion::ALLOWED, "reduceMotion"),
            (LogLevel::ALLOWED, "logLevel"),
        ] {
            assert!(!allowed.is_empty(), "{rendered} has no allowed values");
        }

        assert_eq!(ColorSchemePreference::ALLOWED.len(), 4);
        assert_eq!(
            Accent::ALLOWED.len(),
            8,
            "eight fixed Accents, per ADR-0002"
        );
        assert_eq!(LogLevel::ALLOWED.len(), 5);
    }

    #[test]
    fn every_enum_value_serialises_into_its_allowed_list() {
        let cases: Vec<(serde_json::Value, &[&str])> = vec![
            (
                serde_json::to_value(ColorSchemePreference::Darker).unwrap(),
                ColorSchemePreference::ALLOWED,
            ),
            (
                serde_json::to_value(Accent::Emerald).unwrap(),
                Accent::ALLOWED,
            ),
            (
                serde_json::to_value(Density::Compact).unwrap(),
                Density::ALLOWED,
            ),
            (
                serde_json::to_value(ReduceMotion::Off).unwrap(),
                ReduceMotion::ALLOWED,
            ),
            (
                serde_json::to_value(LogLevel::Trace).unwrap(),
                LogLevel::ALLOWED,
            ),
        ];

        for (value, allowed) in cases {
            let text = value.as_str().expect("enums serialise as strings");
            assert!(
                allowed.contains(&text),
                "{text} is missing from its ALLOWED list"
            );
        }
    }

    #[test]
    fn a_redacted_dump_is_a_full_dump_while_nothing_is_marked() {
        let settings = Settings::default();
        assert_eq!(
            settings.redacted_dump(),
            serde_json::to_value(&settings).unwrap()
        );
    }

    #[test]
    fn marking_a_path_removes_exactly_that_field() {
        let mut value = serde_json::json!({
            "sources": { "privateUrl": "https://user:token@example.invalid" },
            "appearance": { "accent": "blue" }
        });

        redact_path(&mut value, "sources.privateUrl");

        assert_eq!(value["sources"]["privateUrl"], REDACTED_PLACEHOLDER);
        assert_eq!(value["appearance"]["accent"], "blue");
    }
}
