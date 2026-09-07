//! Atomic writes, and the single lock every document write passes through.
//!
//! Two problems this file exists for.
//!
//! **A half-written document is worse than an old one.** Writes go to a temp file in
//! the same directory, are fsynced, and are then renamed over the target. A rename
//! within one volume is atomic, so a power cut leaves either the previous file or the
//! new one, never a truncated hybrid.
//!
//! **Windows locks files.** Antivirus scanners, OneDrive and backup agents all take
//! transient exclusive locks, and on this platform that is common rather than exotic.
//! Every write retries with backoff before giving up.

use std::io::Write;
use std::path::Path;
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::time::Duration;

/// Total attempts, including the first. The backoff below sums to about 1.5s, which
/// covers a scanner's window without making a failure feel like a hang.
const MAX_ATTEMPTS: u32 = 5;

fn backoff(attempt: u32) -> Duration {
    Duration::from_millis(50 * (1_u64 << attempt.min(5)))
}

/// **One lock for every document.** Reset-to-defaults, an import and a debounced
/// sidebar-width write can otherwise interleave and produce a file that is neither.
/// Per-document locks would not fix that: the interleaving that matters is between
/// whole operations, not between files.
fn write_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

/// Hold this for the duration of any multi-step document operation — an import that
/// backs up then writes, for instance — not just around the single write.
pub fn lock() -> MutexGuard<'static, ()> {
    // A poisoned lock means some other write panicked. The data on disk is still
    // consistent, because writes are atomic, so carrying on is correct and refusing
    // to would make one panic permanently break saving.
    write_lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Write `bytes` to `path`, atomically, retrying through a transient lock.
///
/// The caller is expected to already hold [`lock`].
pub fn write(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut last_error = None;

    for attempt in 0..MAX_ATTEMPTS {
        match try_write(path, bytes) {
            Ok(()) => return Ok(()),
            Err(error) if is_transient(&error) => {
                log::warn!(
                    target: "wgm::document",
                    "write to {path:?} blocked (attempt {n} of {MAX_ATTEMPTS}): {error}",
                    n = attempt + 1,
                );
                last_error = Some(error);
                std::thread::sleep(backoff(attempt));
            }
            Err(error) => return Err(error),
        }
    }

    Err(last_error.unwrap_or_else(|| std::io::Error::other("write failed with no recorded cause")))
}

fn try_write(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let directory = path.parent().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "document path has no parent",
        )
    })?;

    std::fs::create_dir_all(directory)?;

    // Same directory as the target, so the rename below stays within one volume.
    let mut temp = tempfile::NamedTempFile::new_in(directory)?;
    temp.write_all(bytes)?;
    temp.flush()?;
    // Durability before visibility: without this the rename can land before the
    // contents do, and a power cut leaves an empty file where a good one was.
    temp.as_file().sync_all()?;

    temp.persist(path).map_err(|error| error.error)?;

    Ok(())
}

/// Is this a lock we should wait out, or a real refusal?
fn is_transient(error: &std::io::Error) -> bool {
    // 32 ERROR_SHARING_VIOLATION, 33 ERROR_LOCK_VIOLATION — the two an antivirus
    // scanner or OneDrive produces. `PermissionDenied` covers the mapped cases.
    matches!(error.raw_os_error(), Some(32) | Some(33))
        || error.kind() == std::io::ErrorKind::PermissionDenied
        || error.kind() == std::io::ErrorKind::Interrupted
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_write_creates_missing_directories() {
        let dir = tempfile::tempdir().expect("a temp dir");
        let path = dir.path().join("nested").join("settings.json");

        write(&path, b"{}").expect("write must succeed");

        assert_eq!(std::fs::read(&path).expect("read"), b"{}");
    }

    #[test]
    fn a_write_replaces_the_previous_contents_whole() {
        let dir = tempfile::tempdir().expect("a temp dir");
        let path = dir.path().join("settings.json");

        write(&path, b"{\"a\":1,\"b\":2}").expect("first write");
        write(&path, b"{}").expect("second write");

        assert_eq!(
            std::fs::read_to_string(&path).expect("read"),
            "{}",
            "a shorter document must not leave a tail of the longer one behind"
        );
    }

    #[test]
    fn no_temporary_files_are_left_behind() {
        let dir = tempfile::tempdir().expect("a temp dir");
        write(&dir.path().join("settings.json"), b"{}").expect("write");

        let entries: Vec<_> = std::fs::read_dir(dir.path())
            .expect("read_dir")
            .flatten()
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .collect();

        assert_eq!(entries, vec!["settings.json".to_owned()]);
    }

    #[test]
    fn the_write_lock_is_reentrant_across_sequential_operations() {
        drop(lock());
        drop(lock());
    }

    #[test]
    fn backoff_grows_and_stays_bounded() {
        assert!(backoff(0) < backoff(3));
        assert!(backoff(20) <= Duration::from_millis(1_600));
    }
}
