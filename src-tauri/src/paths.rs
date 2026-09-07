//! Where wgm keeps its data, and what it does when nowhere is writable.
//!
//! Three modes, resolved once at startup and shown in Settings → About:
//!
//! | Mode | Location |
//! | --- | --- |
//! | Portable | `./data/` beside the executable, when a `wgm.portable` marker sits there |
//! | AppData | `%APPDATA%\io.github.valasme.wgm\` |
//! | Ephemeral | nowhere — Settings live in memory and a banner says so |
//!
//! **Writability is probed, not assumed.** A portable install under `Program Files`
//! cannot write beside itself, so it falls back to `%APPDATA%` with a logged warning.
//! And if neither is writable wgm still starts, degraded: refusing to launch because
//! a directory is read-only is a worse outcome than launching without persistence.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use specta::Type;

/// The marker file that turns an installation portable. Its presence beside the
/// executable is the whole switch; the release zip ships one and the installer does
/// not.
pub const PORTABLE_MARKER: &str = "wgm.portable";

/// Matches `identifier` in `tauri.conf.json`.
const APP_DIR: &str = "io.github.valasme.wgm";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StorageMode {
    Portable,
    AppData,
    /// wgm cannot persist. Settings live in memory only and a banner says so. One
    /// name for one state — never "read-only mode" or "safe mode".
    Ephemeral,
}

/// A machine-readable note about how the location was resolved. No English: the
/// catalog in `src/i18n/en.ts` owns the sentence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PathNote {
    /// A `wgm.portable` marker was found but its folder could not be written to.
    PortableFolderReadOnly,
    /// `%APPDATA%` could not be resolved from the environment.
    AppDataUnavailable,
    /// `%APPDATA%` resolved but could not be written to.
    AppDataReadOnly,
}

/// The resolved location, for the lifetime of the run.
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DataPaths {
    pub data_dir: PathBuf,
    pub log_dir: PathBuf,
    pub mode: StorageMode,
    /// Everything that went wrong on the way to this answer. Surfaced in About.
    pub notes: Vec<PathNote>,
}

impl DataPaths {
    /// True when nothing can be persisted. Every write path checks this first.
    pub fn is_ephemeral(&self) -> bool {
        self.mode == StorageMode::Ephemeral
    }

    /// Where a document with this stem lives.
    pub fn document(&self, stem: &str) -> PathBuf {
        self.data_dir.join(format!("{stem}.json"))
    }
}

/// Resolve the data directory, probing each candidate before committing to it.
pub fn resolve() -> DataPaths {
    let mut notes = Vec::new();

    if let Some(portable_dir) = portable_candidate() {
        if probe_writable(&portable_dir) {
            return finish(portable_dir, StorageMode::Portable, notes);
        }
        // A portable install under Program Files. Common, and not the user's fault.
        notes.push(PathNote::PortableFolderReadOnly);
    }

    match appdata_candidate() {
        Some(appdata_dir) => {
            if probe_writable(&appdata_dir) {
                return finish(appdata_dir, StorageMode::AppData, notes);
            }
            notes.push(PathNote::AppDataReadOnly);
        }
        None => notes.push(PathNote::AppDataUnavailable),
    }

    // Ephemeral Mode. The directory is still named so About can show what wgm
    // *would* have used, and so a log file can be attempted there.
    let fallback = appdata_candidate().unwrap_or_else(std::env::temp_dir);
    finish(fallback, StorageMode::Ephemeral, notes)
}

fn finish(data_dir: PathBuf, mode: StorageMode, notes: Vec<PathNote>) -> DataPaths {
    DataPaths {
        // The log plugin points at the same resolved directory the documents use, so
        // a portable install never splits its state across two locations.
        log_dir: data_dir.join("logs"),
        data_dir,
        mode,
        notes,
    }
}

/// `./data/` beside the executable, but only when the marker file is present.
fn portable_candidate() -> Option<PathBuf> {
    let executable = std::env::current_exe().ok()?;
    let beside = executable.parent()?;
    beside
        .join(PORTABLE_MARKER)
        .is_file()
        .then(|| beside.join("data"))
}

fn appdata_candidate() -> Option<PathBuf> {
    // `%APPDATA%` is roaming, which is what we want: Settings are meaningful on any
    // machine the user signs into. Workspace State is not, but it is small enough
    // that splitting the two across roaming and local would cost more than it saves.
    std::env::var_os("APPDATA").map(|appdata| PathBuf::from(appdata).join(APP_DIR))
}

/// Create the directory and actually write a file into it. Checking permissions any
/// other way is a guess: virtualised folders, redirected profiles and per-user
/// installs under `Program Files` all report the wrong thing.
fn probe_writable(dir: &Path) -> bool {
    if std::fs::create_dir_all(dir).is_err() {
        return false;
    }

    let probe = dir.join(".wgm-write-probe");
    let written = std::fs::write(&probe, b"wgm").is_ok();
    let _ = std::fs::remove_file(&probe);
    written
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_writable_directory_probes_true_and_leaves_nothing_behind() {
        let dir = tempfile::tempdir().expect("a temp dir");
        let target = dir.path().join("nested").join("deeper");

        assert!(probe_writable(&target));
        assert!(
            target.is_dir(),
            "the probe must create the directory it tests"
        );
        assert!(
            !target.join(".wgm-write-probe").exists(),
            "the probe file must be cleaned up"
        );
    }

    #[test]
    fn resolution_always_produces_a_usable_answer() {
        let paths = resolve();

        assert!(paths.log_dir.starts_with(&paths.data_dir));
        assert_eq!(
            paths.document("settings").file_name().unwrap(),
            "settings.json"
        );
    }

    #[test]
    fn ephemeral_mode_is_reported_rather_than_hidden() {
        let paths = DataPaths {
            data_dir: PathBuf::from("C:\\nowhere"),
            log_dir: PathBuf::from("C:\\nowhere\\logs"),
            mode: StorageMode::Ephemeral,
            notes: vec![PathNote::AppDataReadOnly],
        };

        assert!(paths.is_ephemeral());
    }

    #[test]
    fn path_notes_carry_no_english() {
        let wire = serde_json::to_string(&PathNote::PortableFolderReadOnly).expect("serialise");
        assert_eq!(wire, "\"PORTABLE_FOLDER_READ_ONLY\"");
    }
}
