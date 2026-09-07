//! `AppError` — the only error shape that crosses the IPC boundary.
//!
//! Two rules decide everything in this file.
//!
//! **No English crosses the boundary.** An error carries a stable machine `code`,
//! structured `context`, whether it is `recoverable`, and a `correlation_id`. The
//! human sentence lives in `src/i18n/en.ts` and nowhere else. The developer-facing
//! `detail` string is deliberately `#[serde(skip)]`: it exists for the log line.
//!
//! **Errors are logged at construction.** By the time an error reaches the UI, the
//! call site that knew what was being attempted is gone. `ErrorBuilder::emit` is the
//! only way to produce an `AppError`, and it logs and records the problem on the way
//! out — so there is no path that produces an unlogged error.

use std::collections::BTreeMap;
use std::fmt::{self, Display};

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::ring;

/// Stable, machine-readable failure codes.
///
/// Serialised as `SCREAMING_SNAKE_CASE`, which is what appears in a log line, in a
/// Diagnostics Bundle and in `src/i18n/en.ts`. **Never rename a variant**: a user's
/// old log file and a maintainer's grep both depend on these strings outliving the
/// release that produced them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Type)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    // --- Storage -----------------------------------------------------------
    /// The data directory could not be resolved or created anywhere.
    StorageUnavailable,
    /// A document file exists but could not be read.
    DocumentReadFailed,
    /// A document could not be written after the retry budget was spent.
    DocumentWriteFailed,
    /// A document's JSON could not be parsed. The original is backed up, never deleted.
    DocumentCorrupt,
    /// A document declares a `schemaVersion` this build does not know about.
    DocumentSchemaTooNew,
    /// A migration returned an error or panicked. Treated exactly as corruption.
    DocumentMigrationFailed,
    /// A write was attempted while running in Ephemeral Mode.
    StorageEphemeral,

    // --- Import and export -------------------------------------------------
    /// An import file failed validation. Nothing was applied.
    ImportInvalid,
    /// An import file came from a newer schema than this build understands.
    ImportSchemaTooNew,
    /// The pre-import backup could not be written, so the import was aborted.
    ImportBackupFailed,
    /// An export target could not be written.
    ExportFailed,

    // --- Diagnostics -------------------------------------------------------
    /// The Diagnostics Bundle could not be assembled or written.
    DiagnosticsBundleFailed,
    /// Redaction could not determine the current username, so nothing was written.
    DiagnosticsRedactionUnsafe,

    // --- The one network call ----------------------------------------------
    ReleaseCheckOffline,
    ReleaseCheckServerError,
    ReleaseCheckRateLimited,
    ReleaseCheckTimeout,
    ReleaseCheckMalformed,

    // --- Platform ----------------------------------------------------------
    /// A Windows API call the UI depends on failed.
    WindowOperationFailed,
    /// `launch on startup` could not be changed.
    AutostartFailed,

    // --- Catch-all ---------------------------------------------------------
    /// Anything that did not arrive in our shape: a transport failure, a
    /// serialisation error, or a panic crossing the boundary. A raw string reaching
    /// the UI would mean no translation, no correlation id and no recovery action.
    IpcUnknown,

    /// Produced only by the dev-only `debug_error` command.
    DebugForced,
}

impl ErrorCode {
    /// Every variant, so a test can assert this list and the serde spelling agree.
    /// A new code that is not added here fails `every_code_matches_its_wire_form`.
    pub const ALL: &'static [ErrorCode] = &[
        ErrorCode::StorageUnavailable,
        ErrorCode::DocumentReadFailed,
        ErrorCode::DocumentWriteFailed,
        ErrorCode::DocumentCorrupt,
        ErrorCode::DocumentSchemaTooNew,
        ErrorCode::DocumentMigrationFailed,
        ErrorCode::StorageEphemeral,
        ErrorCode::ImportInvalid,
        ErrorCode::ImportSchemaTooNew,
        ErrorCode::ImportBackupFailed,
        ErrorCode::ExportFailed,
        ErrorCode::DiagnosticsBundleFailed,
        ErrorCode::DiagnosticsRedactionUnsafe,
        ErrorCode::ReleaseCheckOffline,
        ErrorCode::ReleaseCheckServerError,
        ErrorCode::ReleaseCheckRateLimited,
        ErrorCode::ReleaseCheckTimeout,
        ErrorCode::ReleaseCheckMalformed,
        ErrorCode::WindowOperationFailed,
        ErrorCode::AutostartFailed,
        ErrorCode::IpcUnknown,
        ErrorCode::DebugForced,
    ];

    /// The `SCREAMING_SNAKE_CASE` spelling, for log lines and bundle manifests.
    pub fn as_str(self) -> &'static str {
        match self {
            ErrorCode::StorageUnavailable => "STORAGE_UNAVAILABLE",
            ErrorCode::DocumentReadFailed => "DOCUMENT_READ_FAILED",
            ErrorCode::DocumentWriteFailed => "DOCUMENT_WRITE_FAILED",
            ErrorCode::DocumentCorrupt => "DOCUMENT_CORRUPT",
            ErrorCode::DocumentSchemaTooNew => "DOCUMENT_SCHEMA_TOO_NEW",
            ErrorCode::DocumentMigrationFailed => "DOCUMENT_MIGRATION_FAILED",
            ErrorCode::StorageEphemeral => "STORAGE_EPHEMERAL",
            ErrorCode::ImportInvalid => "IMPORT_INVALID",
            ErrorCode::ImportSchemaTooNew => "IMPORT_SCHEMA_TOO_NEW",
            ErrorCode::ImportBackupFailed => "IMPORT_BACKUP_FAILED",
            ErrorCode::ExportFailed => "EXPORT_FAILED",
            ErrorCode::DiagnosticsBundleFailed => "DIAGNOSTICS_BUNDLE_FAILED",
            ErrorCode::DiagnosticsRedactionUnsafe => "DIAGNOSTICS_REDACTION_UNSAFE",
            ErrorCode::ReleaseCheckOffline => "RELEASE_CHECK_OFFLINE",
            ErrorCode::ReleaseCheckServerError => "RELEASE_CHECK_SERVER_ERROR",
            ErrorCode::ReleaseCheckRateLimited => "RELEASE_CHECK_RATE_LIMITED",
            ErrorCode::ReleaseCheckTimeout => "RELEASE_CHECK_TIMEOUT",
            ErrorCode::ReleaseCheckMalformed => "RELEASE_CHECK_MALFORMED",
            ErrorCode::WindowOperationFailed => "WINDOW_OPERATION_FAILED",
            ErrorCode::AutostartFailed => "AUTOSTART_FAILED",
            ErrorCode::IpcUnknown => "IPC_UNKNOWN",
            ErrorCode::DebugForced => "DEBUG_FORCED",
        }
    }
}

impl Display for ErrorCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// The error every command returns. See the module docs for the two rules it obeys.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppError {
    pub code: ErrorCode,
    /// Structured, translatable interpolation parameters — `{ "path": "..." }`.
    /// Values are already redacted where they name a filesystem location.
    pub context: BTreeMap<String, String>,
    /// Whether the user can do something other than restart.
    pub recoverable: bool,
    /// Shared with the log lines that produced this failure, so a user quoting
    /// `a3f9c1` can be found in a Diagnostics Bundle. Displayed, not just logged.
    pub correlation_id: String,
    /// Developer-facing English. Never crosses the boundary; it is the log line.
    #[serde(skip)]
    #[specta(skip)]
    pub detail: String,
}

impl Display for AppError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} [{}]: {}",
            self.code, self.correlation_id, self.detail
        )
    }
}

impl std::error::Error for AppError {}

impl AppError {
    /// Start building an error. The only way to finish is [`ErrorBuilder::emit`],
    /// which logs — so an unlogged `AppError` cannot be constructed.
    ///
    /// Returning a builder rather than `Self` is the entire point: if `new` produced
    /// an `AppError` directly, "errors are logged at construction" would be a
    /// convention instead of a property of the type.
    #[allow(clippy::new_ret_no_self)]
    pub fn new(code: ErrorCode) -> ErrorBuilder {
        ErrorBuilder {
            code,
            context: BTreeMap::new(),
            recoverable: true,
            detail: String::new(),
            target: "wgm",
        }
    }

    /// Normalise anything that is not already an `AppError`.
    pub fn from_unknown(detail: impl Display) -> Self {
        AppError::new(ErrorCode::IpcUnknown)
            .detail(detail.to_string())
            .unrecoverable()
            .emit()
    }
}

/// Builder for [`AppError`]. See [`AppError::new`].
#[must_use = "an ErrorBuilder does nothing until .emit() is called, which is what logs it"]
pub struct ErrorBuilder {
    code: ErrorCode,
    context: BTreeMap<String, String>,
    recoverable: bool,
    detail: String,
    target: &'static str,
}

impl ErrorBuilder {
    /// Add one structured context value. These are the interpolation parameters
    /// `t()` receives on the TypeScript side, so the keys are part of the contract.
    pub fn with(mut self, key: &str, value: impl Display) -> Self {
        self.context.insert(key.to_owned(), value.to_string());
        self
    }

    /// Developer-facing English for the log line. Never sent to the webview.
    pub fn detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = detail.into();
        self
    }

    /// Mark the failure as one the user cannot work around in place.
    pub fn unrecoverable(mut self) -> Self {
        self.recoverable = false;
        self
    }

    /// Override the log target, which appears in the `[module]` column.
    pub fn target(mut self, target: &'static str) -> Self {
        self.target = target;
        self
    }

    /// Finish the error: assign a correlation id, write the log line, record it in
    /// the Recent problems ring, and return the value.
    pub fn emit(self) -> AppError {
        let correlation_id = new_correlation_id();

        let context = self
            .context
            .iter()
            .map(|(key, value)| format!("{key}={value:?}"))
            .collect::<Vec<_>>()
            .join(" ");

        // The fixed format from docs/error-reporting.md §2. The timestamp, level and
        // module columns are prepended by the formatter in `logging.rs`; an error
        // record adds the correlation id and code as the first two message tokens.
        log::error!(
            target: self.target,
            "{correlation_id}  {code}  {context}{separator}msg={detail:?}",
            code = self.code,
            separator = if context.is_empty() { "" } else { " " },
            detail = self.detail,
        );

        ring::push_problem(self.code, &correlation_id, &self.context);

        AppError {
            code: self.code,
            context: self.context,
            recoverable: self.recoverable,
            correlation_id,
            detail: self.detail,
        }
    }
}

/// Short, quotable, and unique enough within one run. A user reading `a3f9c1` off a
/// screen and typing it into an issue is the entire design constraint.
pub fn new_correlation_id() -> String {
    uuid::Uuid::new_v4().simple().to_string()[..6].to_owned()
}

/// Every command in wgm returns this.
pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_code_matches_its_wire_form() {
        for code in ErrorCode::ALL {
            let wire = serde_json::to_value(code).expect("an ErrorCode must serialise");
            assert_eq!(
                serde_json::Value::String(code.as_str().to_owned()),
                wire,
                "as_str() and the serde spelling disagree for {code:?}"
            );
        }
    }

    /// Catches a variant added to the enum but not to `ALL`, which would otherwise
    /// let the test above pass while ignoring the new code entirely.
    #[test]
    fn the_all_list_is_complete() {
        let mut sorted = ErrorCode::ALL.to_vec();
        sorted.sort();
        sorted.dedup();
        assert_eq!(
            sorted.len(),
            ErrorCode::ALL.len(),
            "ErrorCode::ALL has a duplicate"
        );
        assert_eq!(
            ErrorCode::ALL.last(),
            Some(&ErrorCode::DebugForced),
            "DebugForced is declared last in the enum; a new variant after it must be \
             appended to ErrorCode::ALL as well"
        );
    }

    #[test]
    fn no_english_crosses_the_boundary() {
        let error = AppError::new(ErrorCode::DocumentCorrupt)
            .with("path", "settings.json")
            .detail("expected value at line 1 column 2")
            .emit();

        let wire = serde_json::to_string(&error).expect("AppError must serialise");

        assert!(wire.contains("DOCUMENT_CORRUPT"));
        assert!(wire.contains("settings.json"));
        assert!(
            !wire.contains("expected value"),
            "the developer-facing detail must not reach the webview: {wire}"
        );
    }

    #[test]
    fn correlation_ids_are_short_and_distinct() {
        let first = AppError::new(ErrorCode::DebugForced).emit();
        let second = AppError::new(ErrorCode::DebugForced).emit();

        assert_eq!(first.correlation_id.len(), 6);
        assert_ne!(first.correlation_id, second.correlation_id);
    }

    #[test]
    fn emitting_an_error_records_it_as_a_recent_problem() {
        let error = AppError::new(ErrorCode::ExportFailed)
            .with("path", "D:\\out.zip")
            .emit();

        let problems = ring::recent_problems();
        let recorded = problems
            .iter()
            .find(|problem| problem.correlation_id == error.correlation_id)
            .expect("emit() must record the problem so Settings can show it");

        assert_eq!(recorded.code, ErrorCode::ExportFailed);
    }

    #[test]
    fn unknown_errors_are_normalised_rather_than_passed_through() {
        let error = AppError::from_unknown("a raw string from somewhere else");

        assert_eq!(error.code, ErrorCode::IpcUnknown);
        assert!(!error.recoverable);
    }
}
