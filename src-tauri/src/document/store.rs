//! An in-memory document with a file behind it.
//!
//! Generic over [`VersionedDocument`], and used by both Settings and Workspace State.
//! Two consumers is what proves the module is genuinely generic rather than a Settings
//! loader with a type parameter bolted on; Kit Files will be the third.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::RwLock;

use serde::Serialize;
use specta::Type;

use crate::document::{self, LoadOutcome, VersionedDocument};
use crate::error::{AppError, AppResult, ErrorCode};

/// What the UI needs to know about a document's health, without knowing its shape.
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DocumentStatus {
    pub outcome: LoadOutcome,
    /// True while this document cannot be persisted — Ephemeral Mode.
    pub ephemeral: bool,
    /// Consecutive failed writes. **Coalesced into one banner, not one toast per
    /// toggle**: instant apply means every switch writes, and a file locked by an
    /// antivirus scanner would otherwise produce a wall of toasts.
    pub consecutive_write_failures: u32,
    /// Fields the file carried that this build does not know. Logged and ignored.
    pub unknown_fields: Vec<String>,
    /// Values that were out of range and were pulled back in.
    pub clamped_fields: Vec<String>,
}

pub struct Store<D: VersionedDocument> {
    path: PathBuf,
    value: RwLock<D>,
    outcome: RwLock<LoadOutcome>,
    unknown_fields: RwLock<Vec<String>>,
    clamped_fields: RwLock<Vec<String>>,
    ephemeral: AtomicBool,
    consecutive_write_failures: AtomicU32,
}

impl<D: VersionedDocument> Store<D> {
    /// Load the document, repairing whatever needs repairing, and rewrite the file if
    /// the load changed anything.
    ///
    /// `storage_ephemeral` is the answer from `paths.rs`: when there is nowhere
    /// writable, the store starts ephemeral regardless of what the file said.
    pub fn open(directory: &Path, storage_ephemeral: bool) -> Self {
        let path = directory.join(format!("{}.json", D::STEM));
        let loaded = document::load::<D>(&path);

        // A too-new file must not be overwritten; nothing else forces this on.
        let ephemeral = storage_ephemeral || loaded.outcome.forces_ephemeral();

        let store = Store {
            path,
            value: RwLock::new(loaded.value),
            outcome: RwLock::new(loaded.outcome.clone()),
            unknown_fields: RwLock::new(loaded.unknown_fields),
            clamped_fields: RwLock::new(loaded.clamped_fields),
            ephemeral: AtomicBool::new(ephemeral),
            consecutive_write_failures: AtomicU32::new(0),
        };

        // Rewrite when the load repaired something, so the next launch is clean and
        // the warning is not repeated forever. Never after SchemaTooNew.
        let should_rewrite = matches!(
            loaded.outcome,
            LoadOutcome::Missing
                | LoadOutcome::FilledFromDefaults { .. }
                | LoadOutcome::Migrated { .. }
                | LoadOutcome::Corrupt { .. }
        );

        if should_rewrite && !ephemeral {
            let value = store.get();
            // A failed repair-write is not fatal: the in-memory document is correct
            // and the next successful write will persist it.
            let _ = store.persist(&value);
        }

        store
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// A snapshot. Documents are small; cloning avoids holding a lock across IPC.
    pub fn get(&self) -> D {
        self.read().clone()
    }

    pub fn is_ephemeral(&self) -> bool {
        self.ephemeral.load(Ordering::Relaxed)
    }

    pub fn status(&self) -> DocumentStatus {
        DocumentStatus {
            outcome: self
                .outcome
                .read()
                .map(|outcome| outcome.clone())
                .unwrap_or(LoadOutcome::Loaded),
            ephemeral: self.is_ephemeral(),
            consecutive_write_failures: self.consecutive_write_failures.load(Ordering::Relaxed),
            unknown_fields: self
                .unknown_fields
                .read()
                .map(|f| f.clone())
                .unwrap_or_default(),
            clamped_fields: self
                .clamped_fields
                .read()
                .map(|f| f.clone())
                .unwrap_or_default(),
        }
    }

    /// Apply a change and persist it.
    ///
    /// **On a failed write the in-memory value is rolled back**, so what the UI reads
    /// back is what is actually on disk. Instant apply promises the control reflects
    /// reality; leaving the new value in memory after a failed write would make that
    /// a lie the next read repeats.
    pub fn update<F>(&self, change: F) -> AppResult<D>
    where
        F: FnOnce(&mut D),
    {
        let previous = self.get();

        let candidate = {
            let mut next = previous.clone();
            change(&mut next);
            next
        };

        if self.is_ephemeral() {
            // Still apply in memory: the user's session should behave normally. The
            // banner already tells them nothing is being saved.
            *self.write() = candidate.clone();
            return Ok(candidate);
        }

        match self.persist(&candidate) {
            Ok(()) => {
                *self.write() = candidate.clone();
                self.consecutive_write_failures.store(0, Ordering::Relaxed);
                Ok(candidate)
            }
            Err(error) => {
                *self.write() = previous;
                self.consecutive_write_failures
                    .fetch_add(1, Ordering::Relaxed);
                Err(error)
            }
        }
    }

    /// Replace the document wholesale — an import, or Reset to defaults.
    pub fn replace(&self, value: D) -> AppResult<D> {
        self.update(|current| *current = value)
    }

    /// Adopt a value that has *already* been written to disk.
    ///
    /// Exists for the import path, which holds the shared write lock across a backup
    /// and a save. `std::sync::Mutex` is not reentrant, so that path cannot go through
    /// [`Store::update`] — it would deadlock on a lock it already owns.
    pub fn replace_in_memory(&self, value: D) {
        *self.write() = value;
        self.consecutive_write_failures.store(0, Ordering::Relaxed);
    }

    /// Adopt a value that could **not** be written, for this session only.
    ///
    /// The write-failure count is deliberately left alone: the write really did fail,
    /// the banner should say so, and the only thing being overridden is the rollback.
    /// Used where losing the change is worse than keeping an unpersisted one — finishing
    /// onboarding, where a rollback would send the user straight back into setup.
    pub fn adopt_unpersisted(&self, value: D) {
        *self.write() = value;
    }

    /// Write the current value to disk without changing it. Used after a repair.
    fn persist(&self, value: &D) -> AppResult<()> {
        if self.is_ephemeral() {
            return Err(AppError::new(ErrorCode::StorageEphemeral)
                .with("document", D::STEM)
                .detail("refusing to write while in Ephemeral Mode")
                .emit());
        }

        document::save(&self.path, value)
    }

    fn read(&self) -> std::sync::RwLockReadGuard<'_, D> {
        self.value
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn write(&self) -> std::sync::RwLockWriteGuard<'_, D> {
        self.value
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use serde_json::{Map, Value};

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase", default)]
    struct Fake {
        schema_version: u32,
        count: u32,
    }

    impl Default for Fake {
        fn default() -> Self {
            Fake {
                schema_version: 2,
                count: 5,
            }
        }
    }

    impl VersionedDocument for Fake {
        const STEM: &'static str = "fake";
        const CURRENT_VERSION: u32 = 2;
        const EXPORTABLE: bool = true;

        fn defaults() -> Self {
            Fake::default()
        }

        fn migrate_step(value: &mut Map<String, Value>, from: u32) -> Result<(), String> {
            match from {
                0 => {
                    value.insert("schemaVersion".into(), Value::from(1));
                    Ok(())
                }
                1 => {
                    // A real rename, so the chain is exercised rather than stubbed.
                    if let Some(old) = value.remove("tally") {
                        value.insert("count".into(), old);
                    }
                    value.insert("schemaVersion".into(), Value::from(2));
                    Ok(())
                }
                other => Err(format!("no step from {other}")),
            }
        }

        fn clamp(value: &mut Map<String, Value>) -> Vec<String> {
            if crate::document::clamp_u32(value, "count", 0..=100) {
                vec!["count".to_owned()]
            } else {
                Vec::new()
            }
        }
    }

    fn temp() -> tempfile::TempDir {
        tempfile::tempdir().expect("a temp dir")
    }

    #[test]
    fn a_missing_file_starts_from_defaults_and_is_written() {
        let dir = temp();
        let store = Store::<Fake>::open(dir.path(), false);

        assert_eq!(store.get(), Fake::default());
        assert!(
            store.path().exists(),
            "a first run must leave a file behind"
        );
    }

    #[test]
    fn an_update_persists_and_is_readable_again() {
        let dir = temp();
        let store = Store::<Fake>::open(dir.path(), false);

        store
            .update(|fake| fake.count = 42)
            .expect("update must succeed");

        let reopened = Store::<Fake>::open(dir.path(), false);
        assert_eq!(reopened.get().count, 42);
    }

    #[test]
    fn a_multi_step_migration_chain_runs_in_order() {
        let dir = temp();
        std::fs::write(dir.path().join("fake.json"), r#"{"tally": 9}"#).expect("write fixture");

        let store = Store::<Fake>::open(dir.path(), false);

        assert_eq!(store.get().count, 9, "the v1 rename must have been applied");
        assert_eq!(store.get().schema_version, 2);
        assert!(matches!(
            store.status().outcome,
            LoadOutcome::Migrated { from: 0 }
        ));
    }

    #[test]
    fn ephemeral_storage_applies_changes_in_memory_but_writes_nothing() {
        let dir = temp();
        let store = Store::<Fake>::open(dir.path(), true);

        store
            .update(|fake| fake.count = 7)
            .expect("an in-memory change still succeeds");

        assert_eq!(store.get().count, 7);
        assert!(
            !store.path().exists(),
            "Ephemeral Mode must not create a file"
        );
    }

    #[test]
    fn a_newer_schema_forces_ephemeral_and_leaves_the_file_untouched() {
        let dir = temp();
        let path = dir.path().join("fake.json");
        let original = r#"{"schemaVersion": 99, "count": 3, "somethingNew": true}"#;
        std::fs::write(&path, original).expect("write fixture");

        let store = Store::<Fake>::open(dir.path(), false);

        assert!(store.is_ephemeral(), "wgm must not guess at a newer schema");
        assert_eq!(store.get(), Fake::default());
        assert_eq!(
            std::fs::read_to_string(&path).expect("read"),
            original,
            "a file from a newer build must never be overwritten"
        );
    }

    #[test]
    fn a_corrupt_file_is_preserved_and_defaults_are_used() {
        let dir = temp();
        let path = dir.path().join("fake.json");
        std::fs::write(&path, "{ this is not json").expect("write fixture");

        let store = Store::<Fake>::open(dir.path(), false);

        assert_eq!(store.get(), Fake::default());

        let LoadOutcome::Corrupt { backup } = store.status().outcome else {
            panic!("an unparseable file must report Corrupt");
        };
        assert!(
            backup.exists(),
            "the corrupt file must be preserved, never deleted"
        );
        assert_eq!(
            std::fs::read_to_string(&backup).expect("read"),
            "{ this is not json"
        );
    }

    #[test]
    fn an_out_of_range_value_is_clamped_rather_than_rejecting_the_file() {
        let dir = temp();
        std::fs::write(
            dir.path().join("fake.json"),
            r#"{"schemaVersion":2,"count":5000}"#,
        )
        .expect("write fixture");

        let store = Store::<Fake>::open(dir.path(), false);

        assert_eq!(store.get().count, 100);
        assert_eq!(store.status().clamped_fields, vec!["count".to_owned()]);
    }

    #[test]
    fn unknown_fields_are_ignored_and_reported_rather_than_failing() {
        let dir = temp();
        std::fs::write(
            dir.path().join("fake.json"),
            r#"{"schemaVersion":2,"count":1,"fromANewerBuild":true}"#,
        )
        .expect("write fixture");

        let store = Store::<Fake>::open(dir.path(), false);

        assert_eq!(store.get().count, 1);
        assert_eq!(
            store.status().unknown_fields,
            vec!["fromANewerBuild".to_owned()]
        );
    }

    #[test]
    fn missing_fields_are_filled_from_defaults_and_the_file_is_rewritten() {
        let dir = temp();
        let path = dir.path().join("fake.json");
        std::fs::write(&path, r#"{"schemaVersion":2}"#).expect("write fixture");

        let store = Store::<Fake>::open(dir.path(), false);

        assert_eq!(store.get().count, 5);
        assert!(matches!(
            store.status().outcome,
            LoadOutcome::FilledFromDefaults { .. }
        ));
        assert!(
            std::fs::read_to_string(&path)
                .expect("read")
                .contains("count"),
            "the repaired document must be written back"
        );
    }

    /// The onboarding case: losing the change would send the user straight back into
    /// setup, so the value is kept for the session even though it was not written — and
    /// the failure count is *not* cleared, because the write really did fail and the
    /// banner should still say so.
    #[test]
    fn an_unpersisted_value_is_adopted_without_hiding_the_write_failure() {
        let dir = temp();
        let store = Store::<Fake>::open(dir.path(), false);

        store.consecutive_write_failures.store(2, Ordering::Relaxed);

        let mut value = store.get();
        value.count = 77;
        store.adopt_unpersisted(value);

        assert_eq!(store.get().count, 77);
        assert_eq!(
            store.status().consecutive_write_failures,
            2,
            "adopting an unpersisted value must not clear the failure count"
        );

        // And it really is unpersisted: the file still holds the previous value.
        let reopened = Store::<Fake>::open(dir.path(), false);
        assert_eq!(reopened.get().count, Fake::default().count);
    }

    #[test]
    fn a_failing_migration_is_treated_exactly_as_corruption() {
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
        #[serde(rename_all = "camelCase", default)]
        struct Fragile {
            schema_version: u32,
        }

        impl VersionedDocument for Fragile {
            const STEM: &'static str = "fragile";
            const CURRENT_VERSION: u32 = 1;
            const EXPORTABLE: bool = false;

            fn defaults() -> Self {
                Fragile { schema_version: 1 }
            }

            fn migrate_step(_: &mut Map<String, Value>, _: u32) -> Result<(), String> {
                Err("this migration always fails".to_owned())
            }

            fn clamp(_: &mut Map<String, Value>) -> Vec<String> {
                Vec::new()
            }
        }

        let dir = temp();
        std::fs::write(dir.path().join("fragile.json"), r#"{"schemaVersion":0}"#)
            .expect("write fixture");

        let store = Store::<Fragile>::open(dir.path(), false);

        let LoadOutcome::Corrupt { backup } = store.status().outcome else {
            panic!("a failed migration must be treated as corruption");
        };
        assert!(backup.exists());
    }

    #[test]
    fn a_panicking_migration_is_also_treated_as_corruption() {
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
        #[serde(rename_all = "camelCase", default)]
        struct Explosive {
            schema_version: u32,
        }

        impl VersionedDocument for Explosive {
            const STEM: &'static str = "explosive";
            const CURRENT_VERSION: u32 = 1;
            const EXPORTABLE: bool = false;

            fn defaults() -> Self {
                Explosive { schema_version: 1 }
            }

            fn migrate_step(_: &mut Map<String, Value>, _: u32) -> Result<(), String> {
                panic!("migrations are supposed to be total, and this one is not");
            }

            fn clamp(_: &mut Map<String, Value>) -> Vec<String> {
                Vec::new()
            }
        }

        let dir = temp();
        std::fs::write(dir.path().join("explosive.json"), r#"{"schemaVersion":0}"#)
            .expect("write fixture");

        // The panic hook prints to stderr; silence it so the test output stays
        // readable. The panic is the assertion, not a failure.
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let store = Store::<Explosive>::open(dir.path(), false);
        std::panic::set_hook(previous);

        assert!(matches!(
            store.status().outcome,
            LoadOutcome::Corrupt { .. }
        ));
    }
}
