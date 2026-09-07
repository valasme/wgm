//! **Copy summary** — the low-friction path, and the one most reports will use.
//!
//! Most users will not attach a zip, so the cheap path is first-class rather than a
//! fallback. This produces a markdown block that pastes straight into a GitHub issue
//! with no file handling at all. The crash screen's *Copy diagnostics* produces the
//! same thing, and it runs through the same redaction — it is the more likely of the
//! two to end up in a public issue.

use crate::diagnostics::{ClientEnvironment, Redactor, ReportSubject};
use crate::logging;
use crate::paths::DataPaths;

/// Build the clipboard block.
pub fn build(
    paths: &DataPaths,
    client: &ClientEnvironment,
    subject: &ReportSubject,
    redactor: &Redactor,
) -> String {
    let mut out = String::new();

    out.push_str("### wgm diagnostics\n\n");
    out.push_str("| | |\n| --- | --- |\n");

    let mut row = |label: &str, value: &str| {
        if !value.is_empty() {
            out.push_str(&format!("| {label} | `{value}` |\n"));
        }
    };

    row("wgm", env!("CARGO_PKG_VERSION"));
    row("commit", logging::build_commit());
    row("built", logging::build_date());
    row("tauri", tauri::VERSION);
    row("webview2", &client.webview_version);
    row("os", &os_build());
    row("scaling", &client.display_scaling);
    row("window", &client.window_size);
    row("scheme", &client.color_scheme);
    row(
        "high contrast",
        if client.high_contrast { "on" } else { "off" },
    );
    row("locale", &client.locale);
    row("timezone", &client.timezone);
    row("install", install_kind(paths));
    row("session", logging::session_id());

    if let Some(code) = &subject.error_code {
        row("error", code);
    }
    if let Some(id) = &subject.correlation_id {
        row("correlation id", id);
    }

    // The data directory can contain the username, so it goes through redaction like
    // everything else. A summary is pasted in public more often than a bundle is.
    out.push_str(&format!(
        "\n<sub>data: `{}`</sub>\n",
        redactor.redact_path(&paths.data_dir)
    ));

    redactor.redact(&out)
}

/// Installed or portable. Says which of the two data-directory stories applies.
pub fn install_kind(paths: &DataPaths) -> &'static str {
    match paths.mode {
        crate::paths::StorageMode::Portable => "portable",
        crate::paths::StorageMode::AppData => "installed",
        crate::paths::StorageMode::Ephemeral => "ephemeral",
    }
}

/// A short OS description. Deliberately not a full `ver` dump: the build number is
/// what matters and the rest is noise in a table.
pub fn os_build() -> String {
    format!("{} {}", std::env::consts::OS, std::env::consts::ARCH)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paths::StorageMode;
    use std::path::PathBuf;

    fn paths() -> DataPaths {
        DataPaths {
            data_dir: PathBuf::from("C:\\Users\\alice\\AppData\\Roaming\\io.github.valasme.wgm"),
            log_dir: PathBuf::from(
                "C:\\Users\\alice\\AppData\\Roaming\\io.github.valasme.wgm\\logs",
            ),
            mode: StorageMode::AppData,
            notes: Vec::new(),
        }
    }

    fn redactor() -> Redactor {
        Redactor::new(
            Some(PathBuf::from("C:\\Users\\alice")),
            Some("alice".to_owned()),
            Some("ALICE-PC".to_owned()),
        )
    }

    #[test]
    fn a_summary_carries_the_correlation_id_the_user_will_quote() {
        let subject = ReportSubject {
            correlation_id: Some("a3f9c1".to_owned()),
            error_code: Some("DOCUMENT_WRITE_FAILED".to_owned()),
        };

        let summary = build(
            &paths(),
            &ClientEnvironment::default(),
            &subject,
            &redactor(),
        );

        assert!(summary.contains("a3f9c1"));
        assert!(summary.contains("DOCUMENT_WRITE_FAILED"));
    }

    #[test]
    fn a_summary_never_contains_the_username() {
        let summary = build(
            &paths(),
            &ClientEnvironment::default(),
            &ReportSubject::default(),
            &redactor(),
        );

        assert!(
            !summary.contains("alice"),
            "the summary is pasted in public more often than a bundle: {summary}"
        );
        assert!(summary.contains("%USERPROFILE%"));
    }

    #[test]
    fn a_summary_with_no_specific_failure_is_still_useful() {
        let summary = build(
            &paths(),
            &ClientEnvironment::default(),
            &ReportSubject::default(),
            &redactor(),
        );

        assert!(summary.contains(env!("CARGO_PKG_VERSION")));
        assert!(summary.contains("installed"));
        assert!(summary.contains(crate::logging::session_id()));
    }

    #[test]
    fn empty_client_fields_are_omitted_rather_than_shown_blank() {
        let summary = build(
            &paths(),
            &ClientEnvironment::default(),
            &ReportSubject::default(),
            &redactor(),
        );

        assert!(!summary.contains("| webview2 |"));
    }
}
