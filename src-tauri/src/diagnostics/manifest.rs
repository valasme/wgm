//! `manifest.txt` and `system.txt` — the two files that make the rest of a bundle
//! interpretable.
//!
//! The manifest states what is in the bundle **and what was truncated**, so a
//! maintainer knows a partial log is partial rather than assuming the app went quiet.
//! It also records the exact version, commit SHA and build id, so they can check out
//! the right source and match the right `.pdb`.

use crate::diagnostics::{ClientEnvironment, Redactor, ReportSubject};
use crate::logging;
use crate::paths::DataPaths;

/// One file that will go into the zip.
#[derive(Debug, Clone)]
pub struct Entry {
    pub name: String,
    pub bytes: usize,
    /// Set when the file was cut down to fit the size cap.
    pub truncated: bool,
}

pub fn manifest(
    entries: &[Entry],
    subject: &ReportSubject,
    redactor: &Redactor,
    notes: &[String],
) -> String {
    let mut out = String::new();

    out.push_str("wgm diagnostics bundle\n");
    out.push_str("======================\n\n");

    out.push_str(&format!("wgm version   {}\n", env!("CARGO_PKG_VERSION")));
    out.push_str(&format!("commit        {}\n", logging::build_commit()));
    out.push_str(&format!("built         {}\n", logging::build_date()));
    out.push_str(&format!("build id      {}\n", build_id()));
    out.push_str(&format!("session id    {}\n", logging::session_id()));
    out.push_str(&format!("generated     {}\n", logging::timestamp()));

    if let Some(id) = &subject.correlation_id {
        out.push_str(&format!("reporting     {id}\n"));
    }
    if let Some(code) = &subject.error_code {
        out.push_str(&format!("error code    {code}\n"));
    }

    out.push_str("\ncontents\n--------\n");
    for entry in entries {
        out.push_str(&format!(
            "{name:<28} {bytes:>10} bytes{note}\n",
            name = entry.name,
            bytes = entry.bytes,
            note = if entry.truncated { "  (TRUNCATED)" } else { "" },
        ));
    }

    if entries.iter().any(|entry| entry.truncated) {
        out.push_str(
            "\nSome files were truncated to keep this bundle within its size cap.\n\
             Oldest log data is dropped first.\n",
        );
    }

    out.push_str("\nredaction\n---------\n");
    out.push_str(
        "The user profile path, the bare username and the machine name are replaced\n\
         throughout. Environment variables are never collected.\n",
    );
    if !redactor.is_safe() {
        out.push_str(
            "\nWARNING: the current username could not be determined, so redaction\n\
             could not be verified. This bundle was NOT written.\n",
        );
    }

    if !notes.is_empty() {
        out.push_str("\nnotes\n-----\n");
        for note in notes {
            out.push_str(&format!("- {note}\n"));
        }
    }

    redactor.redact(&out)
}

/// `system.txt`. Everything a rendering or layout bug turns out to depend on.
pub fn system(paths: &DataPaths, client: &ClientEnvironment, redactor: &Redactor) -> String {
    let mut out = String::new();

    out.push_str("system\n======\n\n");
    out.push_str(&format!(
        "os               {}\n",
        super::summary::os_build()
    ));
    out.push_str(&format!("tauri            {}\n", tauri::VERSION));
    out.push_str(&format!("webview2         {}\n", client.webview_version));
    out.push_str(&format!("display scaling  {}\n", client.display_scaling));
    out.push_str(&format!("window size      {}\n", client.window_size));
    out.push_str(&format!("color scheme     {}\n", client.color_scheme));
    out.push_str(&format!(
        "high contrast    {}\n",
        if client.high_contrast { "on" } else { "off" }
    ));
    out.push_str(&format!("locale           {}\n", client.locale));
    // Restated here because log timestamps carry a local offset, and a maintainer
    // comparing a user's log against their own otherwise loses an hour to arithmetic.
    out.push_str(&format!("timezone         {}\n", client.timezone));
    out.push_str(&format!(
        "install kind     {}\n",
        super::summary::install_kind(paths)
    ));
    out.push_str(&format!(
        "data directory   {}\n",
        redactor.redact_path(&paths.data_dir)
    ));
    out.push_str(&format!(
        "ephemeral mode   {}\n",
        if paths.is_ephemeral() { "yes" } else { "no" }
    ));

    if !paths.notes.is_empty() {
        out.push_str(&format!("path notes       {:?}\n", paths.notes));
    }

    // Deliberately absent: the environment block. Not filtered, not allow-listed —
    // not collected. See docs/error-reporting.md §6.
    out.push_str("\nEnvironment variables are deliberately not collected.\n");

    redactor.redact(&out)
}

/// Identifies the binary for PDB matching. The commit is the practical key; a real
/// PE build id would need the linker's output, which is not available from here.
pub fn build_id() -> String {
    format!("{}+{}", env!("CARGO_PKG_VERSION"), logging::build_commit())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paths::StorageMode;
    use std::path::PathBuf;

    fn redactor() -> Redactor {
        Redactor::new(
            Some(PathBuf::from("C:\\Users\\alice")),
            Some("alice".to_owned()),
            Some("ALICE-PC".to_owned()),
        )
    }

    fn paths(mode: StorageMode) -> DataPaths {
        DataPaths {
            data_dir: PathBuf::from("C:\\Users\\alice\\AppData\\Roaming\\io.github.valasme.wgm"),
            log_dir: PathBuf::from("C:\\Users\\alice\\logs"),
            mode,
            logging_available: true,
            notes: Vec::new(),
        }
    }

    #[test]
    fn the_manifest_states_truncation_rather_than_hiding_it() {
        let entries = vec![
            Entry {
                name: "wgm.log".into(),
                bytes: 100,
                truncated: true,
            },
            Entry {
                name: "system.txt".into(),
                bytes: 40,
                truncated: false,
            },
        ];

        let text = manifest(&entries, &ReportSubject::default(), &redactor(), &[]);

        assert!(text.contains("(TRUNCATED)"));
        assert!(text.contains("Oldest log data is dropped first"));
    }

    #[test]
    fn the_manifest_names_the_failure_being_reported() {
        let subject = ReportSubject {
            correlation_id: Some("a3f9c1".to_owned()),
            error_code: Some("DOCUMENT_CORRUPT".to_owned()),
        };

        let text = manifest(&[], &subject, &redactor(), &[]);

        assert!(text.contains("reporting     a3f9c1"));
        assert!(text.contains("DOCUMENT_CORRUPT"));
    }

    #[test]
    fn the_manifest_lets_a_maintainer_find_the_right_source_and_pdb() {
        let text = manifest(&[], &ReportSubject::default(), &redactor(), &[]);

        assert!(text.contains(env!("CARGO_PKG_VERSION")));
        assert!(text.contains("build id"));
        assert!(text.contains("session id"));
    }

    #[test]
    fn system_txt_collects_no_environment_variables() {
        let text = system(
            &paths(StorageMode::AppData),
            &ClientEnvironment::default(),
            &redactor(),
        );

        assert!(text.contains("Environment variables are deliberately not collected"));
        assert!(!text.contains("PATH="));
    }

    #[test]
    fn system_txt_states_the_install_kind_and_ephemeral_state() {
        let installed = system(
            &paths(StorageMode::AppData),
            &ClientEnvironment::default(),
            &redactor(),
        );
        assert!(installed.contains("install kind     installed"));
        assert!(installed.contains("ephemeral mode   no"));

        let ephemeral = system(
            &paths(StorageMode::Ephemeral),
            &ClientEnvironment::default(),
            &redactor(),
        );
        assert!(ephemeral.contains("ephemeral mode   yes"));
    }

    #[test]
    fn neither_file_contains_the_username() {
        let manifest_text = manifest(&[], &ReportSubject::default(), &redactor(), &[]);
        let system_text = system(
            &paths(StorageMode::Portable),
            &ClientEnvironment::default(),
            &redactor(),
        );

        assert!(!manifest_text.contains("alice"));
        assert!(!system_text.contains("alice"));
    }

    #[test]
    fn an_unsafe_redactor_is_declared_in_the_manifest() {
        let unsafe_redactor = Redactor::new(None, None, None);

        let text = manifest(&[], &ReportSubject::default(), &unsafe_redactor, &[]);

        assert!(text.contains("WARNING"));
        assert!(text.contains("NOT written"));
    }
}
