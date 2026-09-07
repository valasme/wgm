//! Assembling the Diagnostics Bundle, and the preview step that comes first.
//!
//! **The preview is not a nicety.** wgm promises that nothing leaves the machine
//! without the user choosing. Handing them an opaque zip full of their settings and
//! file paths is not that promise kept — it is that promise asserted. So the bundle
//! is built in memory, every file is listed with its size and its contents are
//! readable, and only then is anything written.

use std::io::Write as _;
use std::path::Path;

use serde::Serialize;
use specta::Type;

use crate::diagnostics::{manifest, redact::Redactor, ClientEnvironment, ReportSubject};
use crate::error::{AppError, AppResult, ErrorCode};
use crate::paths::DataPaths;
use crate::ring;
use crate::settings::Settings;

/// Total uncompressed budget. A Trace-level session can outgrow any day-based
/// retention in hours, so the bundle caps itself independently of retention.
const MAX_BUNDLE_BYTES: usize = 12 * 1024 * 1024;

/// One file, as the preview dialog shows it.
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct BundleFile {
    pub name: String,
    /// u32 rather than usize: specta refuses to emit a BigInt into the bindings, and
    /// a bundle file larger than 4 GB is not a case worth typing for.
    pub bytes: u32,
    pub truncated: bool,
    /// The whole file when it is small enough to read, so the user can actually check
    /// the redaction rather than take it on trust. `None` for the large log files.
    pub preview: Option<String>,
}

/// What [`build`] produces: everything the zip will contain, before it exists.
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct BundlePreview {
    pub files: Vec<BundleFile>,
    pub total_bytes: u32,
    /// The name the zip would be saved under.
    pub suggested_name: String,
    /// False when the username could not be determined. The UI must refuse to save.
    pub redaction_safe: bool,
    /// Machine-readable notes about what was dropped or could not be read.
    pub notes: Vec<String>,
}

/// A file inside a bundle, contents included. Opaque on purpose: the only things a
/// caller may do with one are preview it and write it, and both are functions here.
pub struct Staged {
    name: String,
    contents: Vec<u8>,
    truncated: bool,
    previewable: bool,
}

/// Assemble the whole bundle in memory.
///
/// Nothing is written here. [`write`] takes the same staged contents and produces the
/// zip, so the preview and the artefact cannot disagree.
pub fn build(
    paths: &DataPaths,
    settings: &Settings,
    client: &ClientEnvironment,
    subject: &ReportSubject,
) -> (BundlePreview, Vec<Staged>) {
    let redactor = Redactor::from_environment();
    let mut notes = Vec::new();
    let mut staged: Vec<Staged> = Vec::new();

    // system.txt and the redacted Settings dump are small and are the two files a
    // user is most likely to want to read before sending. Both are previewable.
    staged.push(Staged {
        name: "system.txt".to_owned(),
        contents: manifest::system(paths, client, &redactor).into_bytes(),
        truncated: false,
        previewable: true,
    });

    let settings_dump = serde_json::to_string_pretty(&settings.redacted_dump())
        .unwrap_or_else(|error| format!("could not serialise settings: {error}"));
    staged.push(Staged {
        name: "settings.json".to_owned(),
        contents: redactor.redact(&settings_dump).into_bytes(),
        truncated: false,
        previewable: true,
    });

    // The ring buffer. Frequently the only record of an intermittent failure, because
    // it is captured regardless of the configured file level.
    let trace = ring::trace_tail()
        .iter()
        .map(|record| {
            format!(
                "{ts}  {level:<5}  [{target}]  {message}",
                ts = record.timestamp,
                level = record.level,
                target = record.target,
                message = record.message,
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    staged.push(Staged {
        name: "trace-tail.log".to_owned(),
        contents: redactor.redact(&trace).into_bytes(),
        truncated: false,
        previewable: false,
    });

    // Recent problems, so the ten failures the user saw are in the bundle even if
    // retention has already dropped the log lines behind them.
    let problems = ring::recent_problems()
        .iter()
        .map(|problem| {
            format!(
                "{ts}  {code}  {id}  {context:?}",
                ts = problem.timestamp,
                code = problem.code,
                id = problem.correlation_id,
                context = problem.context,
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    staged.push(Staged {
        name: "recent-problems.log".to_owned(),
        contents: redactor.redact(&problems).into_bytes(),
        truncated: false,
        previewable: true,
    });

    // Log files, newest first, until the budget runs out.
    let mut budget = MAX_BUNDLE_BYTES.saturating_sub(staged.iter().map(|f| f.contents.len()).sum());
    for (name, contents, truncated) in
        collect_logs(&paths.log_dir, &redactor, &mut budget, &mut notes)
    {
        staged.push(Staged {
            name,
            contents,
            truncated,
            previewable: false,
        });
    }

    // Corrupt backups: the exact artefact needed to debug a parse failure.
    for (name, contents) in collect_corrupt(&paths.data_dir, &redactor, &mut budget, &mut notes) {
        staged.push(Staged {
            name,
            contents,
            truncated: false,
            previewable: true,
        });
    }

    let entries: Vec<manifest::Entry> = staged
        .iter()
        .map(|file| manifest::Entry {
            name: file.name.clone(),
            bytes: file.contents.len(),
            truncated: file.truncated,
        })
        .collect();

    let manifest_text = manifest::manifest(&entries, subject, &redactor, &notes);
    staged.insert(
        0,
        Staged {
            name: "manifest.txt".to_owned(),
            contents: manifest_text.into_bytes(),
            truncated: false,
            previewable: true,
        },
    );

    let files = staged
        .iter()
        .map(|file| BundleFile {
            name: file.name.clone(),
            bytes: file.contents.len().min(u32::MAX as usize) as u32,
            truncated: file.truncated,
            preview: file
                .previewable
                .then(|| String::from_utf8_lossy(&file.contents).into_owned()),
        })
        .collect();

    let total: usize = staged.iter().map(|file| file.contents.len()).sum();

    let preview = BundlePreview {
        total_bytes: total.min(u32::MAX as usize) as u32,
        files,
        suggested_name: suggested_name(),
        redaction_safe: redactor.is_safe(),
        notes,
    };

    (preview, staged)
}

/// Write the staged contents as a zip.
///
/// Fails rather than leaving a partial `.zip` behind: the temp-file-then-rename in
/// `document::atomic` gives that for free.
pub fn write(target: &Path, staged: &[Staged], redaction_safe: bool) -> AppResult<()> {
    if !redaction_safe {
        // Say so rather than shipping an unredacted bundle.
        return Err(AppError::new(ErrorCode::DiagnosticsRedactionUnsafe)
            .detail("the current username could not be determined")
            .unrecoverable()
            .emit());
    }

    let mut buffer = std::io::Cursor::new(Vec::new());
    {
        let mut zip = zip::ZipWriter::new(&mut buffer);
        let options: zip::write::FileOptions<'_, ()> =
            zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        for file in staged {
            zip.start_file(&file.name, options).map_err(bundle_failed)?;
            zip.write_all(&file.contents).map_err(|error| {
                AppError::new(ErrorCode::DiagnosticsBundleFailed)
                    .with("file", &file.name)
                    .detail(error.to_string())
                    .emit()
            })?;
        }

        zip.finish().map_err(bundle_failed)?;
    }

    let _guard = crate::document::atomic::lock();
    crate::document::atomic::write(target, &buffer.into_inner()).map_err(|error| {
        AppError::new(ErrorCode::DiagnosticsBundleFailed)
            .with("path", target.display())
            .detail(error.to_string())
            .emit()
    })
}

fn bundle_failed(error: zip::result::ZipError) -> AppError {
    AppError::new(ErrorCode::DiagnosticsBundleFailed)
        .detail(error.to_string())
        .emit()
}

/// `wgm-diagnostics-0.1.0-2026-09-07T1422.zip`
pub fn suggested_name() -> String {
    let stamp = crate::logging::timestamp();
    let compact: String = stamp
        .chars()
        .take(16)
        .filter(|c| *c != ':' && *c != '-')
        .collect();
    format!(
        "wgm-diagnostics-{}-{compact}.zip",
        env!("CARGO_PKG_VERSION")
    )
}

fn collect_logs(
    log_dir: &Path,
    redactor: &Redactor,
    budget: &mut usize,
    notes: &mut Vec<String>,
) -> Vec<(String, Vec<u8>, bool)> {
    let Ok(entries) = std::fs::read_dir(log_dir) else {
        notes.push("LOG_DIRECTORY_UNREADABLE".to_owned());
        return Vec::new();
    };

    let mut files: Vec<(std::path::PathBuf, std::time::SystemTime)> = entries
        .flatten()
        .filter(|entry| {
            entry
                .path()
                .extension()
                .is_some_and(|extension| extension == "log")
        })
        .filter_map(|entry| Some((entry.path(), entry.metadata().ok()?.modified().ok()?)))
        .collect();

    // Newest first. Oldest logs are dropped first when the budget runs out, and the
    // manifest says so.
    files.sort_by_key(|file| std::cmp::Reverse(file.1));

    let mut collected = Vec::new();

    for (path, _) in files {
        let Some(name) = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
        else {
            continue;
        };

        let Ok(raw) = std::fs::read_to_string(&path) else {
            notes.push(format!("LOG_UNREADABLE:{name}"));
            continue;
        };

        let redacted = redactor.redact(&raw);
        let mut bytes = redacted.into_bytes();
        let mut truncated = false;

        if bytes.len() > *budget {
            // Keep the *tail* of a truncated log: the last lines are where a failure
            // is, and a head-truncated log throws away the only part that matters.
            let keep = *budget;
            if keep == 0 {
                notes.push(format!("LOG_DROPPED:{name}"));
                continue;
            }
            bytes = bytes.split_off(bytes.len() - keep);
            truncated = true;
        }

        *budget = budget.saturating_sub(bytes.len());
        collected.push((name, bytes, truncated));
    }

    collected
}

fn collect_corrupt(
    data_dir: &Path,
    redactor: &Redactor,
    budget: &mut usize,
    notes: &mut Vec<String>,
) -> Vec<(String, Vec<u8>)> {
    let Ok(entries) = std::fs::read_dir(data_dir) else {
        return Vec::new();
    };

    let mut collected = Vec::new();

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.contains(".corrupt-") {
            continue;
        }

        let Ok(raw) = std::fs::read_to_string(entry.path()) else {
            notes.push(format!("CORRUPT_BACKUP_UNREADABLE:{name}"));
            continue;
        };

        let bytes = redactor.redact(&raw).into_bytes();
        if bytes.len() > *budget {
            notes.push(format!("CORRUPT_BACKUP_DROPPED:{name}"));
            continue;
        }

        *budget = budget.saturating_sub(bytes.len());
        collected.push((format!("corrupt/{name}"), bytes));
    }

    collected
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paths::StorageMode;

    fn fixture() -> (tempfile::TempDir, DataPaths) {
        let dir = tempfile::tempdir().expect("a temp dir");
        let data_dir = dir.path().to_path_buf();
        let log_dir = data_dir.join("logs");
        std::fs::create_dir_all(&log_dir).expect("create log dir");

        let paths = DataPaths {
            data_dir,
            log_dir,
            mode: StorageMode::AppData,
            notes: Vec::new(),
        };

        (dir, paths)
    }

    #[test]
    fn a_bundle_always_carries_a_manifest_first() {
        let (_dir, paths) = fixture();

        let (preview, staged) = build(
            &paths,
            &Settings::default(),
            &ClientEnvironment::default(),
            &ReportSubject::default(),
        );

        assert_eq!(
            staged.first().expect("a bundle is never empty").name,
            "manifest.txt"
        );
        assert_eq!(preview.files.first().expect("preview").name, "manifest.txt");
    }

    #[test]
    fn the_small_files_are_previewable_and_the_logs_are_not() {
        let (_dir, paths) = fixture();

        let (preview, _) = build(
            &paths,
            &Settings::default(),
            &ClientEnvironment::default(),
            &ReportSubject::default(),
        );

        let file = |name: &str| {
            preview
                .files
                .iter()
                .find(|file| file.name == name)
                .expect("missing {name}")
        };

        assert!(
            file("system.txt").preview.is_some(),
            "the user must be able to read this"
        );
        assert!(file("settings.json").preview.is_some());
        assert!(
            file("trace-tail.log").preview.is_none(),
            "a log is too large to preview"
        );
    }

    #[test]
    fn corrupt_backups_are_collected_because_they_are_the_evidence() {
        let (_dir, paths) = fixture();
        std::fs::write(
            paths
                .data_dir
                .join("settings.corrupt-2026-09-07T14-22-31Z.json"),
            "{ not json",
        )
        .expect("write");

        let (preview, _) = build(
            &paths,
            &Settings::default(),
            &ClientEnvironment::default(),
            &ReportSubject::default(),
        );

        assert!(
            preview
                .files
                .iter()
                .any(|file| file.name.starts_with("corrupt/")),
            "{:?}",
            preview.files
        );
    }

    #[test]
    fn logs_are_redacted_on_the_way_in() {
        let (_dir, paths) = fixture();
        let profile = crate::diagnostics::Redactor::from_environment();
        let Some(profile_dir) = profile.profile_dir().cloned() else {
            return; // No identity on this machine; the redaction tests cover the rest.
        };
        std::fs::write(
            paths.log_dir.join("wgm.log"),
            format!("opened {}\\settings.json\n", profile_dir.display()),
        )
        .expect("write");

        let (_, staged) = build(
            &paths,
            &Settings::default(),
            &ClientEnvironment::default(),
            &ReportSubject::default(),
        );

        let log = staged
            .iter()
            .find(|file| file.name == "wgm.log")
            .expect("the log is staged");
        let text = String::from_utf8_lossy(&log.contents);

        assert!(text.contains("%USERPROFILE%"), "{text}");
        assert!(!text.contains(&profile_dir.display().to_string()));
    }

    #[test]
    fn a_bundle_is_not_written_when_redaction_cannot_be_verified() {
        let (dir, paths) = fixture();
        let (_, staged) = build(
            &paths,
            &Settings::default(),
            &ClientEnvironment::default(),
            &ReportSubject::default(),
        );

        let target = dir.path().join("out.zip");
        let error = write(&target, &staged, false).expect_err("an unsafe bundle must be refused");

        assert_eq!(error.code, ErrorCode::DiagnosticsRedactionUnsafe);
        assert!(!target.exists(), "no partial zip may be left behind");
    }

    #[test]
    fn a_written_bundle_is_a_readable_zip_containing_every_staged_file() {
        let (dir, paths) = fixture();
        let (preview, staged) = build(
            &paths,
            &Settings::default(),
            &ClientEnvironment::default(),
            &ReportSubject::default(),
        );

        let target = dir.path().join(&preview.suggested_name);
        write(&target, &staged, true).expect("write must succeed");

        let file = std::fs::File::open(&target).expect("open");
        let mut archive = zip::ZipArchive::new(file).expect("a valid zip");

        let names: Vec<String> = (0..archive.len())
            .map(|index| archive.by_index(index).expect("entry").name().to_owned())
            .collect();

        for staged_file in &staged {
            assert!(
                names.contains(&staged_file.name),
                "{} is missing",
                staged_file.name
            );
        }
    }

    #[test]
    fn the_suggested_name_carries_the_version_and_a_timestamp() {
        let name = suggested_name();

        assert!(name.starts_with("wgm-diagnostics-"));
        assert!(name.contains(env!("CARGO_PKG_VERSION")));
        assert!(name.ends_with(".zip"));
        assert!(
            !name.contains(':'),
            "a colon is not legal in a Windows filename: {name}"
        );
    }

    /// The check from `docs/error-reporting.md` §9 that catches redaction regressions
    /// no reviewer would spot. CI runs this on a real machine with a real username.
    #[test]
    fn no_file_in_a_generated_bundle_contains_the_username() {
        let (_dir, paths) = fixture();

        let redactor = Redactor::from_environment();
        let Some(profile_dir) = redactor.profile_dir().cloned() else {
            panic!("could not determine an identity to check against");
        };
        let Some(username) = profile_dir
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
        else {
            panic!("could not determine a username to check against");
        };

        // Seed the bundle with material that contains the username in several shapes.
        std::fs::write(
            paths.log_dir.join("wgm.log"),
            format!(
                "{}\\x\n{}/y\nuser={username}\n",
                profile_dir.display(),
                profile_dir.display()
            ),
        )
        .expect("write");

        let (_, staged) = build(
            &paths,
            &Settings::default(),
            &ClientEnvironment::default(),
            &ReportSubject::default(),
        );

        let profile = profile_dir.display().to_string().to_ascii_lowercase();

        for file in &staged {
            let text = String::from_utf8_lossy(&file.contents).to_ascii_lowercase();

            assert!(
                !text.contains(&profile),
                "{} leaks the profile path",
                file.name
            );

            // As a *whole word*, not as a substring: on a machine whose account is the
            // Windows default `User`, every `%USERPROFILE%` in the bundle contains the
            // username as a substring and a naive check would fail on correct output.
            assert!(
                !contains_word(&text, &username.to_ascii_lowercase()),
                "{} leaks the username",
                file.name
            );
        }
    }

    fn contains_word(haystack: &str, needle: &str) -> bool {
        let is_word = |character: char| character.is_ascii_alphanumeric() || character == '_';

        haystack.match_indices(needle).any(|(start, matched)| {
            let end = start + matched.len();
            !haystack[..start].ends_with(is_word) && !haystack[end..].starts_with(is_word)
        })
    }
}
