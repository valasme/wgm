//! The generic Versioned Document module.
//!
//! One implementation of load, validate, migrate, save, import and export, shared by
//! Settings and Workspace State today and by Kit Files next. ADR-0001 explains why
//! this exists rather than `tauri-plugin-store`: an untyped key-value blob has no
//! schema, no `schemaVersion` and no migration path, so renaming a setting would
//! silently break every existing user's config.
//!
//! The resilience table in `docs/failure-modes.md` §2 is implemented in [`load`], and
//! every row of it has a fixture test.

pub mod atomic;
pub mod store;

pub use store::{DocumentStatus, Store};

use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use specta::Type;

use crate::error::{AppError, AppResult, ErrorCode};

/// A document wgm owns. Implemented once per file, not once per field.
pub trait VersionedDocument: Serialize + DeserializeOwned + Clone + Send + Sync + 'static {
    /// `settings` produces `settings.json`, `settings.backup.json`,
    /// `settings.corrupt-<ts>.json`.
    const STEM: &'static str;

    /// The schema this build writes.
    const CURRENT_VERSION: u32;

    /// Whether this document may be exported. **Workspace State says no**, and that
    /// is the whole point of the split: the export exclusion is structural rather
    /// than a list someone forgets to update.
    const EXPORTABLE: bool;

    /// The compiled-in baseline. The fallback when a field is missing or a file is
    /// unreadable; never itself persisted as a separate document.
    fn defaults() -> Self;

    /// Migrate raw JSON one version forward. Pure, total, and never panicking — a
    /// panic here is caught and treated exactly as corruption.
    fn migrate_step(value: &mut Map<String, Value>, from: u32) -> Result<(), String>;

    /// Bring out-of-range values back inside their range, returning the names of the
    /// fields that were changed. Clamping rather than rejecting means one bad number
    /// does not cost the user every other setting in the file.
    fn clamp(value: &mut Map<String, Value>) -> Vec<String>;
}

/// What happened when a document was loaded. Surfaced in the UI: a corrupt file
/// produces a persistent banner naming the backup, and a too-new file produces
/// Ephemeral Mode.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum LoadOutcome {
    /// Read and parsed with nothing to report.
    Loaded,
    /// No file yet. First run, or the user deleted it.
    Missing,
    /// Fields were absent and filled from Defaults. The file is rewritten.
    FilledFromDefaults { fields: Vec<String> },
    /// The migration chain ran. The file is rewritten at the new version.
    Migrated { from: u32 },
    /// Unparseable, or a migration failed. The original is copied aside and **never
    /// deleted**; the copy's name is shown to the user.
    Corrupt { backup: PathBuf },
    /// Written by a newer build. wgm does not guess: it boots from Defaults in
    /// Ephemeral Mode and does not overwrite the file.
    SchemaTooNew { found: u32, expected: u32 },
    /// The file exists but could not be read at all.
    Unreadable,
}

impl LoadOutcome {
    /// Should wgm refuse to write this document for the rest of the run?
    ///
    /// Only one outcome says yes. A corrupt file has already been backed up, so
    /// overwriting it is safe; a *newer* file has not, and writing over it would
    /// destroy settings the user's other install still needs.
    pub fn forces_ephemeral(&self) -> bool {
        matches!(self, LoadOutcome::SchemaTooNew { .. })
    }
}

/// A loaded document and the story of how it got here.
#[derive(Debug, Clone)]
pub struct Loaded<D> {
    pub value: D,
    pub outcome: LoadOutcome,
    /// Keys present in the file that this build does not know. Logged, then ignored —
    /// which is what protects a user who downgrades.
    pub unknown_fields: Vec<String>,
    /// Values that were outside their range and were pulled back in.
    pub clamped_fields: Vec<String>,
}

/// Read, validate, migrate and repair a document.
///
/// Never returns an error for a *content* problem: every row of the resilience table
/// ends with a usable value in memory. The only failure this can report is not being
/// able to read the file at all, and even that comes back as `Unreadable` with
/// Defaults rather than as an `Err`.
pub fn load<D: VersionedDocument>(path: &Path) -> Loaded<D> {
    let raw = match std::fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            log::info!(target: "wgm::document", "{stem}: no file yet", stem = D::STEM);
            return plain(D::defaults(), LoadOutcome::Missing);
        }
        Err(error) => {
            AppError::new(ErrorCode::DocumentReadFailed)
                .with("document", D::STEM)
                .detail(format!("{path:?}: {error}"))
                .emit();
            return plain(D::defaults(), LoadOutcome::Unreadable);
        }
    };

    let parsed: Value = match serde_json::from_str(&raw) {
        Ok(parsed) => parsed,
        Err(error) => {
            return corrupt::<D>(path, &format!("parse failed: {error}"));
        }
    };

    let Value::Object(mut object) = parsed else {
        return corrupt::<D>(path, "top level is not an object");
    };

    // No `schemaVersion` means version 0 — a file written before the field existed,
    // or one a user hand-edited. Assuming the current version instead would skip the
    // migration chain on exactly the files that most need it.
    let found_version = object
        .get("schemaVersion")
        .and_then(Value::as_u64)
        .and_then(|version| u32::try_from(version).ok())
        .unwrap_or(0);

    if found_version > D::CURRENT_VERSION {
        // Do not guess, and do not overwrite. The file belongs to a build that knows
        // more than this one does.
        AppError::new(ErrorCode::DocumentSchemaTooNew)
            .with("document", D::STEM)
            .with("found", found_version)
            .with("expected", D::CURRENT_VERSION)
            .detail(format!("{path:?} was written by a newer version of wgm"))
            .emit();
        return plain(
            D::defaults(),
            LoadOutcome::SchemaTooNew {
                found: found_version,
                expected: D::CURRENT_VERSION,
            },
        );
    }

    let migrated = found_version < D::CURRENT_VERSION;
    if migrated {
        // A migration that panics is a migration that failed. Catching it here is
        // what makes "migrations are pure and total" a property rather than a hope.
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            for version in found_version..D::CURRENT_VERSION {
                D::migrate_step(&mut object, version)?;
            }
            Ok::<(), String>(())
        }));

        match outcome {
            Ok(Ok(())) => {
                object.insert("schemaVersion".into(), Value::from(D::CURRENT_VERSION));
            }
            Ok(Err(reason)) => {
                return corrupt::<D>(path, &format!("migration from v{found_version}: {reason}"));
            }
            Err(_) => {
                return corrupt::<D>(path, &format!("migration from v{found_version} panicked"));
            }
        }
    }

    let reference = serde_json::to_value(D::defaults()).unwrap_or(Value::Null);
    let reference_object = reference.as_object().cloned().unwrap_or_default();

    // Unknown keys are logged and left alone; serde ignores them on the way in and
    // they are dropped on the way out. That is deliberate: a user who downgrades,
    // runs the older build once and upgrades again loses the newer settings, but
    // never loses the file.
    let unknown_fields = diff_keys(&object, &reference_object, "");
    if !unknown_fields.is_empty() {
        log::warn!(
            target: "wgm::document",
            "{stem}: ignoring unknown fields {unknown_fields:?}",
            stem = D::STEM,
        );
    }

    let missing_fields = diff_keys(&reference_object, &object, "");
    let clamped_fields = D::clamp(&mut object);
    if !clamped_fields.is_empty() {
        log::warn!(
            target: "wgm::document",
            "{stem}: clamped out-of-range fields {clamped_fields:?}",
            stem = D::STEM,
        );
    }

    // Missing fields are filled by serde's `#[serde(default)]`, which is why the
    // schema declares one on every field.
    let value: D = match serde_json::from_value(Value::Object(object)) {
        Ok(value) => value,
        Err(error) => {
            return corrupt::<D>(path, &format!("shape is not this schema: {error}"));
        }
    };

    let outcome = if migrated {
        log::info!(
            target: "wgm::document",
            "{stem}: migrated from v{found_version}",
            stem = D::STEM,
        );
        LoadOutcome::Migrated {
            from: found_version,
        }
    } else if !missing_fields.is_empty() {
        log::warn!(
            target: "wgm::document",
            "{stem}: filled missing fields {missing_fields:?} from defaults",
            stem = D::STEM,
        );
        LoadOutcome::FilledFromDefaults {
            fields: missing_fields,
        }
    } else {
        LoadOutcome::Loaded
    };

    Loaded {
        value,
        outcome,
        unknown_fields,
        clamped_fields,
    }
}

/// Serialise and write a document atomically.
///
/// Takes the shared write lock, so an import and a debounced sidebar-width write
/// cannot interleave.
pub fn save<D: VersionedDocument>(path: &Path, value: &D) -> AppResult<()> {
    let _guard = atomic::lock();
    save_locked(path, value)
}

/// As [`save`], for callers already holding the lock.
pub fn save_locked<D: VersionedDocument>(path: &Path, value: &D) -> AppResult<()> {
    let bytes = serde_json::to_vec_pretty(value).map_err(|error| {
        AppError::new(ErrorCode::DocumentWriteFailed)
            .with("document", D::STEM)
            .detail(format!("serialise failed: {error}"))
            .emit()
    })?;

    atomic::write(path, &bytes).map_err(|error| {
        AppError::new(ErrorCode::DocumentWriteFailed)
            .with("document", D::STEM)
            .with("path", path.display())
            .detail(format!("{path:?}: {error}"))
            .emit()
    })
}

/// Copy the current file aside before an operation that replaces it wholesale.
///
/// **The backup is the undo.** An import that cannot write one is an import that must
/// not run, which is why this returns a hard error rather than a warning.
pub fn backup<D: VersionedDocument>(path: &Path) -> AppResult<Option<PathBuf>> {
    if !path.exists() {
        return Ok(None);
    }

    let target = path.with_file_name(format!("{}.backup.json", D::STEM));

    std::fs::copy(path, &target)
        .map(|_| Some(target.clone()))
        .map_err(|error| {
            AppError::new(ErrorCode::ImportBackupFailed)
                .with("document", D::STEM)
                .with("path", target.display())
                .detail(format!("could not write {target:?}: {error}"))
                .emit()
        })
}

/// Copy an unreadable file aside under a timestamped name, and **never delete it**.
/// The copy is the exact artefact needed to debug a parse failure, and the user is
/// told its name.
fn corrupt<D: VersionedDocument>(path: &Path, reason: &str) -> Loaded<D> {
    let stamp = crate::logging::timestamp().replace(':', "-");
    let target = path.with_file_name(format!("{}.corrupt-{stamp}.json", D::STEM));

    let backup = match std::fs::copy(path, &target) {
        Ok(_) => target,
        Err(error) => {
            log::error!(
                target: "wgm::document",
                "could not preserve the corrupt {stem} file: {error}",
                stem = D::STEM,
            );
            // Report the path we tried. The banner naming a file that does not exist
            // is still better than silence about a file that was not saved.
            path.with_file_name(format!("{}.corrupt-{stamp}.json", D::STEM))
        }
    };

    AppError::new(ErrorCode::DocumentCorrupt)
        .with("document", D::STEM)
        .with("backup", backup.display())
        .detail(format!("{path:?}: {reason}"))
        .emit();

    Loaded {
        value: D::defaults(),
        outcome: LoadOutcome::Corrupt { backup },
        unknown_fields: Vec::new(),
        clamped_fields: Vec::new(),
    }
}

fn plain<D>(value: D, outcome: LoadOutcome) -> Loaded<D> {
    Loaded {
        value,
        outcome,
        unknown_fields: Vec::new(),
        clamped_fields: Vec::new(),
    }
}

/// Dotted paths present in `candidate` but not in `reference`, recursing through
/// nested objects. Used in both directions: unknown fields one way, missing the other.
fn diff_keys(
    candidate: &Map<String, Value>,
    reference: &Map<String, Value>,
    prefix: &str,
) -> Vec<String> {
    let mut found = Vec::new();

    for (key, value) in candidate {
        let path = if prefix.is_empty() {
            key.clone()
        } else {
            format!("{prefix}.{key}")
        };

        match reference.get(key) {
            None => found.push(path),
            Some(Value::Object(reference_child)) => {
                if let Value::Object(candidate_child) = value {
                    found.extend(diff_keys(candidate_child, reference_child, &path));
                }
            }
            Some(_) => {}
        }
    }

    found
}

/// Clamp helper for the schemas, which all need the same integer behaviour.
///
/// Returns true when the value was changed, so the caller can name the field.
pub fn clamp_u32(
    object: &mut Map<String, Value>,
    key: &str,
    range: std::ops::RangeInclusive<u64>,
) -> bool {
    let Some(current) = object.get(key).and_then(Value::as_u64) else {
        return false;
    };

    let clamped = current.clamp(*range.start(), *range.end());
    if clamped == current {
        return false;
    }

    object.insert(key.to_owned(), Value::from(clamped));
    true
}

/// Reset a string field to a default when it is not one of the allowed values.
///
/// An unrecognised enum would fail deserialisation and cost the user the whole file,
/// which is the outcome the clamp rule exists to avoid.
pub fn clamp_enum(
    object: &mut Map<String, Value>,
    key: &str,
    allowed: &[&str],
    fallback: &str,
) -> bool {
    let Some(current) = object.get(key).and_then(Value::as_str) else {
        return false;
    };

    if allowed.contains(&current) {
        return false;
    }

    object.insert(key.to_owned(), Value::from(fallback));
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diff_keys_finds_nested_additions() {
        let candidate: Map<String, Value> =
            serde_json::from_str(r#"{"a":{"b":1,"c":2},"d":3}"#).expect("parse");
        let reference: Map<String, Value> =
            serde_json::from_str(r#"{"a":{"b":1},"e":4}"#).expect("parse");

        let mut found = diff_keys(&candidate, &reference, "");
        found.sort();

        assert_eq!(found, vec!["a.c".to_owned(), "d".to_owned()]);
    }

    #[test]
    fn clamping_pulls_a_number_back_inside_its_range() {
        let mut object: Map<String, Value> =
            serde_json::from_str(r#"{"days": 9000}"#).expect("parse");

        assert!(clamp_u32(&mut object, "days", 1..=90));
        assert_eq!(object["days"], Value::from(90));
        assert!(
            !clamp_u32(&mut object, "days", 1..=90),
            "a value in range is untouched"
        );
    }

    #[test]
    fn an_unrecognised_enum_falls_back_rather_than_failing_the_file() {
        let mut object: Map<String, Value> =
            serde_json::from_str(r#"{"scheme":"chartreuse"}"#).expect("parse");

        assert!(clamp_enum(
            &mut object,
            "scheme",
            &["light", "dark"],
            "light"
        ));
        assert_eq!(object["scheme"], Value::from("light"));
    }

    #[test]
    fn only_a_newer_schema_forces_ephemeral_mode() {
        assert!(LoadOutcome::SchemaTooNew {
            found: 9,
            expected: 1
        }
        .forces_ephemeral());
        assert!(!LoadOutcome::Corrupt {
            backup: PathBuf::from("x")
        }
        .forces_ephemeral());
        assert!(!LoadOutcome::Loaded.forces_ephemeral());
    }
}
