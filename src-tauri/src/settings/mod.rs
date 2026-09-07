//! Settings — the exportable half of wgm's persisted state.

pub mod commands;
pub mod defaults;
pub mod migrations;
pub mod schema;

use serde_json::{Map, Value};

use crate::document::VersionedDocument;
use crate::error::{AppError, AppResult, ErrorCode};

pub use schema::Settings;

impl VersionedDocument for Settings {
    const STEM: &'static str = "settings";
    const CURRENT_VERSION: u32 = schema::CURRENT_VERSION;
    /// Settings holds everything meaningful on any machine, so all of it travels.
    const EXPORTABLE: bool = true;

    fn defaults() -> Self {
        defaults::defaults()
    }

    fn migrate_step(value: &mut Map<String, Value>, from: u32) -> Result<(), String> {
        migrations::step(value, from)
    }

    fn clamp(value: &mut Map<String, Value>) -> Vec<String> {
        defaults::clamp(value)
    }
}

/// The envelope an exported Settings file is wrapped in.
///
/// Carries enough for the importing build to decide what to do without guessing:
/// the schema version drives migration or refusal, and the other two fields are
/// there for a human reading the file.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct SettingsExport {
    pub schema_version: u32,
    pub exported_at: String,
    pub app_version: String,
    pub settings: Value,
}

/// What an import would do, shown to the user before anything is applied.
#[derive(Debug, Clone, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ImportPreview {
    /// Dotted paths whose value differs from what is in effect now.
    pub changes: Vec<ImportChange>,
    /// True when the file came from an older schema and was migrated to preview it.
    pub migrated: bool,
    pub from_version: u32,
}

#[derive(Debug, Clone, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ImportChange {
    pub path: String,
    pub current: String,
    pub incoming: String,
}

/// Build the export envelope.
pub fn export_envelope(settings: &Settings) -> SettingsExport {
    SettingsExport {
        schema_version: schema::CURRENT_VERSION,
        exported_at: crate::logging::timestamp(),
        app_version: env!("CARGO_PKG_VERSION").to_owned(),
        // Workspace State is not merely omitted here — it is a different document
        // that this function has no access to. The exclusion is structural.
        settings: serde_json::to_value(settings).unwrap_or(Value::Null),
    }
}

/// Validate an import file and produce the Settings it would apply.
///
/// **Rejects atomically.** Nothing is ever partially applied: either every field
/// validates and the whole document is returned, or the file is refused and the
/// current Settings are untouched.
pub fn validate_import(raw: &str) -> AppResult<(Settings, ImportPreview, u32)> {
    let envelope: SettingsExport = serde_json::from_str(raw).map_err(|error| {
        AppError::new(ErrorCode::ImportInvalid)
            .detail(format!("not a wgm settings export: {error}"))
            .emit()
    })?;

    if envelope.schema_version > schema::CURRENT_VERSION {
        // Refuse with an explanation rather than guessing at fields we do not know.
        return Err(AppError::new(ErrorCode::ImportSchemaTooNew)
            .with("found", envelope.schema_version)
            .with("expected", schema::CURRENT_VERSION)
            .detail("the file was exported by a newer version of wgm")
            .emit());
    }

    let Value::Object(mut object) = envelope.settings else {
        return Err(AppError::new(ErrorCode::ImportInvalid)
            .detail("the `settings` member is not an object")
            .emit());
    };

    let from_version = envelope.schema_version;
    let migrated = from_version < schema::CURRENT_VERSION;

    if migrated {
        for version in from_version..schema::CURRENT_VERSION {
            migrations::step(&mut object, version).map_err(|reason| {
                AppError::new(ErrorCode::ImportInvalid)
                    .with("fromVersion", version)
                    .detail(format!("migration failed: {reason}"))
                    .emit()
            })?;
        }
    }

    // Any invalid field rejects the whole file, so clamping is deliberately *not*
    // applied here. Repairing a file wgm wrote is kindness; silently repairing a
    // file the user handed us would hide that it was wrong.
    let settings: Settings = serde_json::from_value(Value::Object(object)).map_err(|error| {
        AppError::new(ErrorCode::ImportInvalid)
            .detail(format!("a field is not valid: {error}"))
            .emit()
    })?;

    Ok((
        settings.clone(),
        ImportPreview {
            changes: Vec::new(),
            migrated,
            from_version,
        },
        from_version,
    ))
}

/// Compare two Settings and describe what would change, for the preview step.
pub fn diff(current: &Settings, incoming: &Settings) -> Vec<ImportChange> {
    let current = serde_json::to_value(current).unwrap_or(Value::Null);
    let incoming = serde_json::to_value(incoming).unwrap_or(Value::Null);

    let mut changes = Vec::new();
    collect_changes(&current, &incoming, "", &mut changes);
    changes
}

fn collect_changes(
    current: &Value,
    incoming: &Value,
    prefix: &str,
    changes: &mut Vec<ImportChange>,
) {
    match (current, incoming) {
        (Value::Object(left), Value::Object(right)) => {
            for (key, incoming_value) in right {
                // `schemaVersion` always differs after a migration and is not a
                // setting anyone chose, so it is not a change worth previewing.
                if prefix.is_empty() && key == "schemaVersion" {
                    continue;
                }
                let path = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{prefix}.{key}")
                };
                match left.get(key) {
                    Some(current_value) => {
                        collect_changes(current_value, incoming_value, &path, changes)
                    }
                    None => changes.push(ImportChange {
                        path,
                        current: String::new(),
                        incoming: render(incoming_value),
                    }),
                }
            }
        }
        (left, right) if left != right => changes.push(ImportChange {
            path: prefix.to_owned(),
            current: render(left),
            incoming: render(right),
        }),
        _ => {}
    }
}

fn render(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::schema::{Accent, ColorSchemePreference};

    #[test]
    fn settings_are_exportable_and_carry_their_version() {
        let envelope = export_envelope(&Settings::default());

        assert_eq!(envelope.schema_version, schema::CURRENT_VERSION);
        assert_eq!(envelope.app_version, env!("CARGO_PKG_VERSION"));
        assert!(envelope.settings.is_object());
    }

    #[test]
    fn an_export_round_trips_through_import() {
        let mut original = Settings::default();
        original.appearance.accent = Accent::Rose;
        original.advanced.log_retention_days = 30;

        let raw = serde_json::to_string(&export_envelope(&original)).expect("serialise");
        let (imported, _, _) = validate_import(&raw).expect("a wgm export must import");

        assert_eq!(imported, original);
    }

    #[test]
    fn a_newer_schema_is_refused_with_a_code_rather_than_guessed_at() {
        let raw = serde_json::json!({
            "schemaVersion": schema::CURRENT_VERSION + 1,
            "exportedAt": "2026-09-07T14:22:31Z",
            "appVersion": "99.0.0",
            "settings": {}
        })
        .to_string();

        let error = validate_import(&raw).expect_err("a newer export must be refused");

        assert_eq!(error.code, ErrorCode::ImportSchemaTooNew);
    }

    #[test]
    fn an_older_schema_is_migrated_and_flagged_as_such() {
        let raw = serde_json::json!({
            "schemaVersion": 0,
            "exportedAt": "2026-09-07T14:22:31Z",
            "appVersion": "0.0.1",
            "settings": { "appearance": { "accent": "cyan" } }
        })
        .to_string();

        let (settings, preview, from) = validate_import(&raw).expect("an older export migrates");

        assert!(preview.migrated);
        assert_eq!(from, 0);
        assert_eq!(settings.appearance.accent, Accent::Cyan);
    }

    #[test]
    fn one_invalid_field_rejects_the_whole_file() {
        let raw = serde_json::json!({
            "schemaVersion": schema::CURRENT_VERSION,
            "exportedAt": "2026-09-07T14:22:31Z",
            "appVersion": "0.1.0",
            "settings": { "appearance": { "accent": "chartreuse" } }
        })
        .to_string();

        let error = validate_import(&raw).expect_err("an invented accent must be refused");

        assert_eq!(error.code, ErrorCode::ImportInvalid);
    }

    #[test]
    fn a_hostile_file_is_refused_rather_than_partially_applied() {
        for raw in [
            "",
            "null",
            "[]",
            "{\"settings\": 3}",
            "{\"schemaVersion\": \"one\"}",
        ] {
            assert!(validate_import(raw).is_err(), "{raw:?} must not import");
        }
    }

    #[test]
    fn the_preview_counts_only_real_changes() {
        let current = Settings::default();
        let mut incoming = Settings::default();
        incoming.appearance.color_scheme = ColorSchemePreference::Darker;
        incoming.general.restore_window_position = false;

        let changes = diff(&current, &incoming);

        assert_eq!(changes.len(), 2, "{changes:?}");
        assert!(changes
            .iter()
            .any(|change| change.path == "appearance.colorScheme"));
        assert!(changes
            .iter()
            .any(|change| change.path == "general.restoreWindowPosition"));
    }

    #[test]
    fn an_identical_import_previews_no_changes() {
        assert!(diff(&Settings::default(), &Settings::default()).is_empty());
    }
}
