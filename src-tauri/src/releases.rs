//! The wgm Release check — **the only outbound request in the entire product**.
//!
//! It runs when the user clicks *Check for a new version* in Settings → About, sends
//! no identifiers, and does nothing but report a version number and a link.
//!
//! It is made **from Rust**, never from the webview. That is what makes ADR-0003
//! structural rather than conventional: the Content-Security-Policy restricts
//! `connect-src` to `'self'` and `ipc:`, so the webview *cannot* reach the network
//! even if someone later writes a `fetch` by mistake. Every outbound request in wgm
//! therefore lives in exactly this one auditable function.
//!
//! Note the vocabulary: this is a **wgm Release**, never an "update". `/updates`
//! already means Package Updates.

use std::time::Duration;

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::error::{AppError, ErrorCode};

/// The unauthenticated GitHub releases endpoint. No token, no identifiers, no
/// telemetry parameters.
const RELEASES_URL: &str = "https://api.github.com/repos/valasme/wgm/releases/latest";

/// Ten seconds, then treated as offline. A check the user is waiting on must not hang.
const TIMEOUT: Duration = Duration::from_secs(10);

/// Every outcome the UI has copy for, per `docs/failure-modes.md` §5.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase", tag = "status")]
pub enum ReleaseCheck {
    /// A newer release exists.
    Newer { version: String, url: String },
    /// Already current.
    UpToDate { version: String },
}

/// What GitHub returns, reduced to the two fields wgm reads.
#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    html_url: String,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    prerelease: bool,
}

/// Ask GitHub for the latest release.
///
/// Returns an [`AppError`] carrying one of the five release-check codes for every
/// failure path, so the UI can say something specific rather than "something went
/// wrong".
pub async fn check() -> Result<ReleaseCheck, AppError> {
    let client = tauri_plugin_http::reqwest::Client::builder()
        .timeout(TIMEOUT)
        // GitHub rejects requests with no user agent. This one names the product and
        // its version and nothing else — no machine id, no install id, no counter.
        .user_agent(concat!("wgm/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|error| {
            AppError::new(ErrorCode::ReleaseCheckOffline)
                .detail(format!("could not build an HTTP client: {error}"))
                .emit()
        })?;

    let response = client
        .get(RELEASES_URL)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|error| {
            let code = if error.is_timeout() {
                ErrorCode::ReleaseCheckTimeout
            } else {
                ErrorCode::ReleaseCheckOffline
            };
            AppError::new(code).detail(error.to_string()).emit()
        })?;

    let status = response.status();

    // The unauthenticated GitHub API allows 60 requests an hour per IP. Telling the
    // user *when it will work again* is the difference between a useful message and
    // a button that mysteriously does nothing.
    if status.as_u16() == 403 || status.as_u16() == 429 {
        let reset_at = response
            .headers()
            .get("x-ratelimit-reset")
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned)
            .unwrap_or_default();

        return Err(AppError::new(ErrorCode::ReleaseCheckRateLimited)
            .with("resetAt", reset_at)
            .detail(format!("GitHub returned {status}"))
            .emit());
    }

    if status.is_server_error() {
        return Err(AppError::new(ErrorCode::ReleaseCheckServerError)
            .with("status", status.as_u16())
            .detail(format!("GitHub returned {status}"))
            .emit());
    }

    if !status.is_success() {
        return Err(AppError::new(ErrorCode::ReleaseCheckMalformed)
            .with("status", status.as_u16())
            .detail(format!("unexpected status {status}"))
            .emit());
    }

    // Read as text and parse here rather than through reqwest's `json` feature: the
    // plugin does not enable it, and one fewer feature on an HTTP client that only
    // ever calls one endpoint is the right trade.
    let body = response.text().await.map_err(|error| {
        AppError::new(ErrorCode::ReleaseCheckMalformed)
            .detail(error.to_string())
            .emit()
    })?;

    let release: GitHubRelease = serde_json::from_str(&body).map_err(|error| {
        AppError::new(ErrorCode::ReleaseCheckMalformed)
            .detail(error.to_string())
            .emit()
    })?;

    if release.draft || release.prerelease {
        return Ok(ReleaseCheck::UpToDate {
            version: env!("CARGO_PKG_VERSION").to_owned(),
        });
    }

    Ok(compare(
        env!("CARGO_PKG_VERSION"),
        &release.tag_name,
        &release.html_url,
    ))
}

/// Decide whether the published tag is newer than what is running.
///
/// Split out from the request so the comparison is testable without a network.
pub fn compare(current: &str, tag: &str, url: &str) -> ReleaseCheck {
    let published = normalise(tag);

    if is_newer(&published, &normalise(current)) {
        ReleaseCheck::Newer {
            version: published,
            url: url.to_owned(),
        }
    } else {
        ReleaseCheck::UpToDate {
            version: current.to_owned(),
        }
    }
}

/// `v0.2.0` and `0.2.0` are the same release.
fn normalise(version: &str) -> String {
    version.trim().trim_start_matches(['v', 'V']).to_owned()
}

/// A numeric comparison, not a string one: `0.10.0` is newer than `0.9.0`, and a
/// lexical compare gets that backwards.
fn is_newer(candidate: &str, current: &str) -> bool {
    parts(candidate) > parts(current)
}

fn parts(version: &str) -> (u64, u64, u64) {
    // Anything after a `-` is a pre-release suffix, which wgm does not publish.
    let core = version.split('-').next().unwrap_or_default();
    let mut numbers = core.split('.').map(|part| part.parse::<u64>().unwrap_or(0));

    (
        numbers.next().unwrap_or(0),
        numbers.next().unwrap_or(0),
        numbers.next().unwrap_or(0),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_higher_version_is_offered() {
        let result = compare("0.1.0", "v0.2.0", "https://example.invalid/r");

        assert_eq!(
            result,
            ReleaseCheck::Newer {
                version: "0.2.0".to_owned(),
                url: "https://example.invalid/r".to_owned()
            }
        );
    }

    #[test]
    fn the_same_version_reports_up_to_date() {
        assert_eq!(
            compare("0.1.0", "v0.1.0", "https://example.invalid/r"),
            ReleaseCheck::UpToDate {
                version: "0.1.0".to_owned()
            }
        );
    }

    #[test]
    fn an_older_published_tag_does_not_offer_a_downgrade() {
        assert_eq!(
            compare("1.2.0", "v1.1.9", "https://example.invalid/r"),
            ReleaseCheck::UpToDate {
                version: "1.2.0".to_owned()
            }
        );
    }

    #[test]
    fn versions_compare_numerically_rather_than_lexically() {
        assert!(is_newer("0.10.0", "0.9.0"), "0.10.0 is newer than 0.9.0");
        assert!(!is_newer("0.9.0", "0.10.0"));
        assert!(is_newer("1.0.0", "0.99.99"));
    }

    #[test]
    fn a_v_prefix_is_not_a_version_difference() {
        assert_eq!(normalise("v1.2.3"), "1.2.3");
        assert_eq!(normalise(" V1.2.3 "), "1.2.3");
    }

    #[test]
    fn a_malformed_tag_does_not_panic_and_does_not_offer_an_upgrade() {
        assert_eq!(
            compare("0.1.0", "not-a-version", "https://example.invalid/r"),
            ReleaseCheck::UpToDate {
                version: "0.1.0".to_owned()
            }
        );
    }

    #[test]
    fn a_prerelease_suffix_is_ignored_in_the_comparison() {
        assert_eq!(parts("1.2.3-rc.1"), (1, 2, 3));
    }

    #[test]
    fn the_endpoint_carries_no_identifiers() {
        assert!(
            !RELEASES_URL.contains('?'),
            "no query parameters: {RELEASES_URL}"
        );
        assert!(RELEASES_URL.starts_with("https://api.github.com/repos/valasme/wgm/"));
    }
}
