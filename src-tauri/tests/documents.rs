//! The document resilience table, driven by checked-in fixtures.
//!
//! `docs/failure-modes.md` §2 is a table of conditions and behaviours. Every row is a
//! test here, and every row runs against **both** documents — Settings and Workspace
//! State — because "the module is generic" is a claim that only means something if
//! both consumers are exercised.
//!
//! Fixtures live in `tests/fixtures/settings/` and `tests/fixtures/workspace/`.

use std::path::{Path, PathBuf};

use wgm_lib::document::{LoadOutcome, Store, VersionedDocument};
use wgm_lib::settings::Settings;
use wgm_lib::workspace::WorkspaceState;

fn fixture(document: &str, name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(document)
        .join(name);

    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("missing fixture {path:?}: {error}"))
}

/// Stage a fixture as `<stem>.json` in a fresh directory and open the store over it.
fn open<D: VersionedDocument>(fixture_name: &str) -> (tempfile::TempDir, PathBuf, Store<D>) {
    let dir = tempfile::tempdir().expect("a temp dir");
    let path = dir.path().join(format!("{}.json", D::STEM));

    std::fs::write(&path, fixture(D::STEM, fixture_name)).expect("stage the fixture");

    let store = Store::<D>::open(dir.path(), false);
    (dir, path, store)
}

/// Every row below runs twice. Writing it as a macro rather than a generic function
/// keeps each row a separate `#[test]`, so a failure names the document *and* the row.
macro_rules! for_both_documents {
    ($name:ident, $body:item) => {
        mod $name {
            use super::*;
            $body

            #[test]
            fn settings() {
                run::<Settings>();
            }

            #[test]
            fn workspace() {
                run::<WorkspaceState>();
            }
        }
    };
}

// --- Unparseable -----------------------------------------------------------

for_both_documents!(
    unparseable_is_backed_up_and_never_deleted,
    fn run<D: VersionedDocument + PartialEq + std::fmt::Debug>() {
        let (_dir, path, store) = open::<D>("corrupt.json");

        assert_eq!(
            store.get(),
            D::defaults(),
            "a corrupt file must boot from Defaults"
        );

        let LoadOutcome::Corrupt { backup } = store.status().outcome else {
            panic!("an unparseable file must report Corrupt");
        };

        assert!(
            backup.exists(),
            "the copy must exist so the banner can name it"
        );
        assert!(
            backup
                .file_name()
                .unwrap()
                .to_string_lossy()
                .contains(".corrupt-"),
            "{backup:?} is not a corrupt backup name"
        );
        assert_eq!(
            std::fs::read_to_string(&backup).expect("read the backup"),
            fixture(D::STEM, "corrupt.json"),
            "the backup must be byte-for-byte what the user had"
        );
        assert!(
            path.exists(),
            "the document itself is rewritten from Defaults"
        );
    }
);

// --- Missing fields --------------------------------------------------------

for_both_documents!(
    missing_fields_are_filled_from_defaults_and_rewritten,
    fn run<D: VersionedDocument + PartialEq + std::fmt::Debug>() {
        let (_dir, path, store) = open::<D>("missing-fields.json");

        let LoadOutcome::FilledFromDefaults { fields } = store.status().outcome else {
            panic!("a document missing fields must report FilledFromDefaults");
        };
        assert!(!fields.is_empty(), "the filled fields must be named");

        let rewritten = std::fs::read_to_string(&path).expect("read");
        let reparsed: serde_json::Value = serde_json::from_str(&rewritten).expect("valid JSON");
        let expected = serde_json::to_value(store.get()).expect("serialise");

        assert_eq!(
            reparsed, expected,
            "the repaired document must be written back"
        );
    }
);

// --- Unknown fields --------------------------------------------------------

for_both_documents!(
    unknown_fields_are_ignored_and_reported,
    fn run<D: VersionedDocument + PartialEq + std::fmt::Debug>() {
        let (_dir, _path, store) = open::<D>("unknown-fields.json");

        let status = store.status();

        assert!(
            status
                .unknown_fields
                .iter()
                .any(|field| field.contains("fromANewerBuild")),
            "the unknown field must be reported: {:?}",
            status.unknown_fields
        );
        assert!(
            !matches!(status.outcome, LoadOutcome::Corrupt { .. }),
            "an unknown field must not cost the user their file — this is what \
             protects someone who downgrades"
        );
    }
);

// --- Out-of-range values ---------------------------------------------------

for_both_documents!(
    out_of_range_values_are_clamped_rather_than_rejected,
    fn run<D: VersionedDocument + PartialEq + std::fmt::Debug>() {
        let (_dir, _path, store) = open::<D>("out-of-range.json");

        let status = store.status();

        assert!(
            !status.clamped_fields.is_empty(),
            "the clamped fields must be named"
        );
        assert!(
            !matches!(status.outcome, LoadOutcome::Corrupt { .. }),
            "one bad number must not cost the user every other value in the file"
        );
    }
);

// --- A newer schemaVersion -------------------------------------------------

for_both_documents!(
    a_newer_schema_boots_from_defaults_and_never_overwrites,
    fn run<D: VersionedDocument + PartialEq + std::fmt::Debug>() {
        let (_dir, path, store) = open::<D>("schema-too-new.json");
        let original = fixture(D::STEM, "schema-too-new.json");

        assert!(
            store.is_ephemeral(),
            "a newer schema must force Ephemeral Mode"
        );
        assert_eq!(store.get(), D::defaults());
        assert!(matches!(
            store.status().outcome,
            LoadOutcome::SchemaTooNew { .. }
        ));
        assert_eq!(
            std::fs::read_to_string(&path).expect("read"),
            original,
            "a file written by a newer build must be left exactly as it was"
        );
    }
);

// --- An older schemaVersion ------------------------------------------------

for_both_documents!(
    an_older_schema_runs_the_migration_chain_and_is_rewritten,
    fn run<D: VersionedDocument + PartialEq + std::fmt::Debug>() {
        let (_dir, path, store) = open::<D>("schema-v0.json");

        assert!(
            matches!(store.status().outcome, LoadOutcome::Migrated { from: 0 }),
            "a file with no schemaVersion is version 0 and must be migrated: {:?}",
            store.status().outcome
        );

        let rewritten: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).expect("read")).expect("JSON");

        assert_eq!(
            rewritten["schemaVersion"],
            serde_json::Value::from(D::CURRENT_VERSION),
            "the migrated document must be rewritten at the current version"
        );
    }
);

// --- Writes ----------------------------------------------------------------

for_both_documents!(
    a_good_document_loads_clean_and_survives_a_round_trip,
    fn run<D: VersionedDocument + PartialEq + std::fmt::Debug>() {
        let (dir, _path, store) = open::<D>("valid.json");

        assert_eq!(store.status().outcome, LoadOutcome::Loaded);
        assert!(store.status().unknown_fields.is_empty());
        assert!(store.status().clamped_fields.is_empty());

        let loaded = store.get();
        let reopened = Store::<D>::open(dir.path(), false);

        assert_eq!(reopened.get(), loaded);
    }
);

for_both_documents!(
    a_missing_file_is_created_from_defaults,
    fn run<D: VersionedDocument + PartialEq + std::fmt::Debug>() {
        let dir = tempfile::tempdir().expect("a temp dir");
        let store = Store::<D>::open(dir.path(), false);

        assert_eq!(store.status().outcome, LoadOutcome::Missing);
        assert_eq!(store.get(), D::defaults());
        assert!(
            store.path().exists(),
            "a first run must leave a file behind"
        );
    }
);

for_both_documents!(
    ephemeral_storage_never_writes_a_file,
    fn run<D: VersionedDocument + PartialEq + std::fmt::Debug>() {
        let dir = tempfile::tempdir().expect("a temp dir");
        let store = Store::<D>::open(dir.path(), true);

        assert!(store.is_ephemeral());
        assert!(
            !store.path().exists(),
            "Ephemeral Mode must not create a file"
        );
    }
);

// --- The split between the two documents -----------------------------------

#[test]
fn workspace_state_is_structurally_excluded_from_export() {
    // Compile-time, because this is the property that keeps window geometry and
    // onboarding progress out of an export. Flipping either constant should fail the
    // build, not a test run.
    const _: () = {
        assert!(!WorkspaceState::EXPORTABLE);
        assert!(Settings::EXPORTABLE);
    };

    assert_ne!(Settings::STEM, WorkspaceState::STEM);
}

#[test]
fn an_export_carries_no_workspace_state() {
    let envelope = wgm_lib::settings::export_envelope(&Settings::default());
    let rendered = serde_json::to_string(&envelope).expect("serialise");

    for forbidden in ["sidebar", "onboarding", "maximized", "completedVersion"] {
        assert!(
            !rendered.contains(forbidden),
            "an export must not carry `{forbidden}`: {rendered}"
        );
    }
}
