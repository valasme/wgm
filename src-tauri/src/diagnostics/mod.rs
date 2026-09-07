//! Diagnostics — the only route by which anything leaves a user's machine, and the
//! user chooses when and to whom.
//!
//! Two artefacts, and the smaller one is the default:
//!
//! - **Summary** — a markdown block on the clipboard. Version, build, Windows build,
//!   WebView2 version, error code and Correlation Id. It pastes straight into a
//!   GitHub issue with no file handling at all, and it is what most bug reports will
//!   actually contain.
//! - **Diagnostics Bundle** — the zip. The escalation, not the default, and it is
//!   never written without showing the user every file in it first.

pub mod bundle;
pub mod manifest;
pub mod redact;
pub mod summary;

use serde::{Deserialize, Serialize};
use specta::Type;

pub use redact::Redactor;

/// Facts only the webview knows, passed in rather than guessed at.
///
/// Rendering and layout bugs are almost always specific to the WebView2 version or
/// the display scaling, so neither is padding.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase", default)]
pub struct ClientEnvironment {
    pub webview_version: String,
    pub display_scaling: String,
    pub locale: String,
    pub timezone: String,
    /// The Color Scheme actually in effect, not the preference.
    pub color_scheme: String,
    pub high_contrast: bool,
    pub window_size: String,
}

/// What is being reported, when a report is about one specific failure.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase", default)]
pub struct ReportSubject {
    pub correlation_id: Option<String>,
    pub error_code: Option<String>,
}
