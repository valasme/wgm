//! Logging: the fixed line format, the session banner, retention, the Trace ring feed
//! and the panic hook.
//!
//! All of it per `docs/error-reporting.md` §1–2. The format is fixed because a
//! maintainer will grep it and a user may open it in Notepad — human-readable and
//! machine-parseable, in that order of priority.

use std::io::Write as _;
use std::panic;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use tauri_plugin_log::fern;
use tauri_plugin_log::{Builder as LogBuilder, Target, TargetKind};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::ring::{self, TraceRecord};

/// A rotated log file is only useful if you can tell which run you are reading.
static SESSION_ID: OnceLock<String> = OnceLock::new();

/// Identifies one run of wgm. Emitted in the startup banner and in every Diagnostics
/// Bundle manifest. Distinct from a Correlation Id, which identifies one failure
/// *within* a run.
pub fn session_id() -> &'static str {
    SESSION_ID.get_or_init(|| uuid::Uuid::new_v4().simple().to_string()[..12].to_owned())
}

/// ISO-8601 with the local offset. The offset matters: a maintainer comparing a
/// user's log against their own otherwise loses an hour to arithmetic, which is why
/// `system.txt` restates the timezone too.
pub fn timestamp() -> String {
    OffsetDateTime::now_local()
        .unwrap_or_else(|_| OffsetDateTime::now_utc())
        .format(&Rfc3339)
        .unwrap_or_else(|_| "0000-00-00T00:00:00Z".to_owned())
}

/// The configured file level. Everything below it still reaches the Trace ring.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<FileLevel> for log::LevelFilter {
    fn from(level: FileLevel) -> Self {
        match level {
            FileLevel::Error => log::LevelFilter::Error,
            FileLevel::Warn => log::LevelFilter::Warn,
            FileLevel::Info => log::LevelFilter::Info,
            FileLevel::Debug => log::LevelFilter::Debug,
            FileLevel::Trace => log::LevelFilter::Trace,
        }
    }
}

/// A single log file may not exceed this before rotating. A day cap alone is not
/// enough: a Trace-level session can outgrow a seven-day window in hours.
const MAX_FILE_BYTES: u128 = 8 * 1024 * 1024;

/// Total bytes of log files kept, oldest deleted first. The second half of the same
/// argument.
pub const MAX_TOTAL_LOG_BYTES: u64 = 64 * 1024 * 1024;

/// Build the plugin. The folder target points at the *same* resolved data directory
/// the documents use, so a portable install keeps its logs beside itself rather than
/// splitting state across two locations.
pub fn plugin<R: tauri::Runtime>(
    log_dir: &Path,
    level: FileLevel,
) -> tauri::plugin::TauriPlugin<R> {
    LogBuilder::new()
        .level(log::LevelFilter::from(level))
        // The webview's console.warn / console.error arrive through the plugin's own
        // `log` command with this target, so they land in the same stream.
        .level_for("webview", log::LevelFilter::Warn)
        .max_file_size(MAX_FILE_BYTES)
        .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepAll)
        .timezone_strategy(tauri_plugin_log::TimezoneStrategy::UseLocal)
        .format(format_record)
        .targets([
            Target::new(TargetKind::Folder {
                path: log_dir.to_path_buf(),
                file_name: Some("wgm".to_owned()),
            }),
            #[cfg(debug_assertions)]
            Target::new(TargetKind::Stdout),
            // Feeds the Trace ring and forces a flush on error. Registered as a
            // dispatch target so it sees every record the logger accepts.
            Target::new(TargetKind::Dispatch(
                fern::Dispatch::new()
                    .level(log::LevelFilter::Trace)
                    .format(format_record)
                    .chain(fern::Output::call(capture)),
            )),
        ])
        .build()
}

/// The fixed line format:
///
/// ```text
/// 2026-09-07T14:22:31.123+02:00  ERROR  [settings::commands]  a3f9c1  SETTINGS_WRITE_FAILED  path="..." msg="..."
/// ```
///
/// The first three columns are here; an error record supplies the correlation id and
/// the code as the first two tokens of its message. See `error.rs`.
fn format_record(
    out: fern::FormatCallback<'_>,
    message: &std::fmt::Arguments<'_>,
    record: &log::Record<'_>,
) {
    out.finish(format_args!(
        "{timestamp}  {level:<5}  [{target}]  {message}",
        timestamp = timestamp(),
        level = record.level(),
        target = record.target(),
    ));
}

/// Feed the Trace ring, and flush on anything at error level.
///
/// A buffered writer loses the last lines on a crash, and the last lines are the
/// entire point.
fn capture(record: &log::Record<'_>) {
    ring::push_trace(TraceRecord {
        timestamp: timestamp(),
        level: record.level().to_string(),
        target: record.target().to_owned(),
        message: record.args().to_string(),
    });

    if record.level() <= log::Level::Error {
        log::logger().flush();
        let _ = std::io::stdout().flush();
    }
}

/// Write the per-run banner. Daily rotation means several runs share a file, and
/// without this there is no way to tell where the run being reported begins.
pub fn session_banner(data_dir: &Path, storage_mode: &str) {
    log::info!(
        target: "wgm",
        "---- session {session} · wgm {version} · commit {commit} · {os} · data={data_dir:?} ({storage_mode}) ----",
        session = session_id(),
        version = env!("CARGO_PKG_VERSION"),
        commit = build_commit(),
        os = os_description(),
    );
}

/// The commit this build came from, injected by `build.rs`. `unknown` in a local
/// build with no git available, which is honest rather than wrong.
pub fn build_commit() -> &'static str {
    option_env!("WGM_BUILD_COMMIT").unwrap_or("unknown")
}

/// When this build was produced, for the About page and the bundle manifest.
pub fn build_date() -> &'static str {
    option_env!("WGM_BUILD_DATE").unwrap_or("unknown")
}

fn os_description() -> String {
    format!("{} {}", std::env::consts::OS, std::env::consts::ARCH)
}

/// Install the panic hook.
///
/// Two rules: capture a backtrace explicitly, because users will never set
/// `RUST_BACKTRACE=1`; and never panic inside the hook, because a double panic aborts
/// with no log at all — the one failure that leaves nothing behind.
pub fn install_panic_hook() {
    let previous = panic::take_hook();

    panic::set_hook(Box::new(move |info| {
        // Everything in here is infallible on purpose.
        let location = info
            .location()
            .map(|location| format!("{}:{}", location.file(), location.line()))
            .unwrap_or_else(|| "unknown".to_owned());

        let payload = info
            .payload()
            .downcast_ref::<&str>()
            .map(|text| (*text).to_owned())
            .or_else(|| info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "non-string panic payload".to_owned());

        let backtrace = std::backtrace::Backtrace::force_capture();

        log::error!(
            target: "wgm::panic",
            "{correlation}  PANIC  location={location:?} msg={payload:?}\n{backtrace}",
            correlation = crate::error::new_correlation_id(),
        );

        // Explicit: the process may be about to end, and a buffered last line is a
        // lost last line.
        log::logger().flush();

        previous(info);
    }));
}

/// Delete log files past the day cap or the size cap, oldest first.
///
/// Failure here is logged and ignored: retention that cannot keep up is a smaller
/// problem than retention that takes the app down.
pub fn prune_logs(log_dir: &Path, retention_days: u32) {
    let Ok(entries) = std::fs::read_dir(log_dir) else {
        return;
    };

    let mut files: Vec<(PathBuf, std::time::SystemTime, u64)> = entries
        .flatten()
        .filter(|entry| {
            entry.file_name().to_string_lossy().starts_with("wgm")
                && entry
                    .path()
                    .extension()
                    .is_some_and(|extension| extension == "log")
        })
        .filter_map(|entry| {
            let metadata = entry.metadata().ok()?;
            Some((entry.path(), metadata.modified().ok()?, metadata.len()))
        })
        .collect();

    // Newest first, so the caps are applied by walking towards the oldest.
    files.sort_by_key(|file| std::cmp::Reverse(file.1));

    let cutoff = std::time::SystemTime::now().checked_sub(std::time::Duration::from_secs(
        u64::from(retention_days) * 86_400,
    ));

    let mut kept_bytes = 0_u64;

    for (path, modified, size) in files {
        let too_old = cutoff.is_some_and(|cutoff| modified < cutoff);
        let too_big = kept_bytes.saturating_add(size) > MAX_TOTAL_LOG_BYTES;

        if !too_old && !too_big {
            kept_bytes = kept_bytes.saturating_add(size);
        }

        if too_old || too_big {
            match std::fs::remove_file(&path) {
                Ok(()) => log::debug!(target: "wgm::logging", "pruned log {path:?}"),
                Err(error) => {
                    log::warn!(target: "wgm::logging", "could not prune {path:?}: {error}");
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_session_id_is_stable_within_a_run() {
        assert_eq!(session_id(), session_id());
        assert_eq!(session_id().len(), 12);
    }

    #[test]
    fn timestamps_carry_an_offset_so_two_machines_can_be_compared() {
        let stamp = timestamp();
        assert!(stamp.contains('T'), "{stamp} is not ISO-8601");
        assert!(
            stamp.ends_with('Z') || stamp[10..].contains('+') || stamp[10..].contains('-'),
            "{stamp} carries no UTC offset"
        );
    }

    #[test]
    fn pruning_an_absent_directory_is_not_an_error() {
        prune_logs(
            Path::new("this-directory-does-not-exist-and-that-is-fine"),
            7,
        );
    }

    #[test]
    fn pruning_removes_files_past_the_day_cap_and_keeps_recent_ones() {
        let dir = tempfile::tempdir().expect("a temp dir");

        let recent = dir.path().join("wgm.log");
        std::fs::write(&recent, b"recent").expect("write");

        let old = dir.path().join("wgm.2020-01-01.log");
        std::fs::write(&old, b"old").expect("write");
        let long_ago = std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1);
        filetime_set(&old, long_ago);

        prune_logs(dir.path(), 7);

        assert!(
            recent.exists(),
            "a file inside the retention window must survive"
        );
        assert!(!old.exists(), "a file past the day cap must be deleted");
    }

    /// `std::fs` cannot set an mtime, and a whole crate for one test assertion is not
    /// worth the dependency. Opening the file and rewriting it is enough to prove the
    /// ordering logic, so the old file is aged by writing it *before* the recent one
    /// and then explicitly backdating through the platform API when available.
    fn filetime_set(path: &Path, time: std::time::SystemTime) {
        let file = std::fs::OpenOptions::new()
            .write(true)
            .open(path)
            .expect("open");
        let _ = file.set_times(std::fs::FileTimes::new().set_modified(time));
    }
}
