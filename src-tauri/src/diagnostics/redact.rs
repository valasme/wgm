//! Redaction — the mandatory, non-optional rewriting of user-identifying strings
//! before anything leaves the machine.
//!
//! **Deny by default, and never a `C:\Users\` regex.** That pattern misses redirected
//! profiles, `D:\Users`, UNC home directories and domain-joined machines entirely.
//! Instead the real profile directory is resolved from the OS, and *that* path is
//! redacted, along with the bare username wherever it appears and the machine name.
//!
//! **Environment variables are never collected.** Not filtered, not allow-listed —
//! not collected. A developer's environment routinely holds `GITHUB_TOKEN`,
//! `NPM_TOKEN` and cloud credentials, and this audience's environment holds more of
//! them than most. The two variables read below are read to *find out what to
//! redact*; neither their names nor their values are ever written into a bundle.
//!
//! **If the username cannot be determined, say so rather than shipping an
//! unredacted bundle.** [`Redactor::is_safe`] is checked before a zip is written.

use std::path::PathBuf;

/// One replacement.
#[derive(Debug, Clone)]
struct Rule {
    needle: String,
    replacement: &'static str,
    /// When true the match must be bounded by non-identifier characters on both
    /// sides. See the comment in [`Redactor::new`] about the `User` account.
    whole_word: bool,
}

impl Rule {
    fn substring(needle: String, replacement: &'static str) -> Self {
        Rule {
            needle,
            replacement,
            whole_word: false,
        }
    }

    fn word(needle: String, replacement: &'static str) -> Self {
        Rule {
            needle,
            replacement,
            whole_word: true,
        }
    }
}

/// Replacements applied to every string that goes into a Diagnostics Bundle.
#[derive(Debug, Clone)]
pub struct Redactor {
    /// Longest first, so `C:\Users\alice\AppData` is replaced before `alice` is.
    rules: Vec<Rule>,
    profile_dir: Option<PathBuf>,
    username_known: bool,
}

impl Redactor {
    /// Build from the OS.
    pub fn from_environment() -> Self {
        let profile_dir = resolve_profile_dir();

        // The profile directory's own file name is the username, and taking it from
        // there rather than from `USERNAME` is what makes redirected profiles and UNC
        // home directories work.
        let username = profile_dir
            .as_ref()
            .and_then(|dir| dir.file_name())
            .map(|name| name.to_string_lossy().into_owned())
            .filter(|name| !name.is_empty())
            .or_else(|| std::env::var("USERNAME").ok())
            .filter(|name| !name.is_empty());

        let computer = std::env::var("COMPUTERNAME")
            .ok()
            .filter(|name| !name.is_empty());

        Self::new(profile_dir, username, computer)
    }

    /// Construct explicitly. Exists so the golden-file test can pin a known identity
    /// rather than depending on whoever is running it.
    pub fn new(
        profile_dir: Option<PathBuf>,
        username: Option<String>,
        computer_name: Option<String>,
    ) -> Self {
        let mut rules: Vec<Rule> = Vec::new();

        if let Some(dir) = &profile_dir {
            let path = dir.to_string_lossy().into_owned();
            // Substring matching, because a path is always surrounded by punctuation
            // and is never a word in running text.
            rules.push(Rule::substring(path.clone(), "%USERPROFILE%"));
            // Forward slashes appear in log lines written by the webview and in JSON.
            rules.push(Rule::substring(path.replace('\\', "/"), "%USERPROFILE%"));
            // JSON-escaped backslashes, which is how a path looks inside settings.json.
            rules.push(Rule::substring(path.replace('\\', "\\\\"), "%USERPROFILE%"));
        }

        // **Whole words only.** The default Windows account is literally called
        // `User`, and substring-matching that would rewrite `%USERPROFILE%` into
        // `%<redacted>PROFILE%` and `user=` into `%USERNAME%=`, destroying the log
        // while claiming to protect it.
        if let Some(name) = &username {
            rules.push(Rule::word(name.clone(), "%USERNAME%"));
        }

        // Machine names are frequently a person's name or an employer's.
        if let Some(name) = &computer_name {
            rules.push(Rule::word(name.clone(), "%COMPUTERNAME%"));
        }

        // Longest first. Without this, redacting `alice` first would turn
        // `C:\Users\alice` into `C:\Users\%USERNAME%` and the profile rule would then
        // never match — leaking the directory layout while looking like it worked.
        rules.sort_by_key(|rule| std::cmp::Reverse(rule.needle.len()));

        Redactor {
            rules,
            profile_dir,
            username_known: username.is_some(),
        }
    }

    /// False when the username could not be determined. A bundle must not be written
    /// in that state; `manifest.txt` says so instead.
    pub fn is_safe(&self) -> bool {
        self.username_known
    }

    pub fn profile_dir(&self) -> Option<&PathBuf> {
        self.profile_dir.as_ref()
    }

    /// Apply every rule. Case-insensitive on Windows paths, because a log line may
    /// carry `c:\users\alice` while the OS reports `C:\Users\Alice`.
    pub fn redact(&self, input: &str) -> String {
        let mut output = input.to_owned();

        for rule in &self.rules {
            if rule.needle.is_empty() {
                continue;
            }
            output =
                replace_ignore_ascii_case(&output, &rule.needle, rule.replacement, rule.whole_word);
        }

        redact_token_shapes(&output)
    }

    /// Convenience for redacting a path for display.
    pub fn redact_path(&self, path: &std::path::Path) -> String {
        self.redact(&path.to_string_lossy())
    }
}

/// The placeholder a redacted secret leaves behind.
pub const TOKEN_PLACEHOLDER: &str = "%TOKEN%";

/// Prefixes that are unambiguously the start of a credential. Deliberately a short,
/// conservative list: a general "looks random" heuristic would eat correlation ids,
/// commit SHAs and package hashes, and a bundle with its identifiers mangled is a
/// bundle nobody can act on.
const SECRET_PREFIXES: &[&str] = &[
    "ghp_",
    "gho_",
    "ghu_",
    "ghs_",
    "ghr_",
    "github_pat_",
    "npm_",
    "xoxb-",
    "xoxp-",
    "sk-",
    "AKIA",
    "ASIA",
    "AIza",
];

/// Characters a token can be made of. A match is a maximal run of these that begins
/// with one of [`SECRET_PREFIXES`], so surrounding quotes, braces and commas survive
/// and the line stays readable.
fn is_token_char(character: char) -> bool {
    character.is_ascii_alphanumeric() || character == '_' || character == '-'
}

/// Blank out anything token-shaped.
///
/// Path redaction cannot catch a credential, and this audience's logs are more likely
/// than most to contain one. This is a second line of defence, not the first: the
/// first is that environment variables are never collected and that Settings fields
/// are marked as redacted individually — see `settings::schema::REDACTED_PATHS`.
fn redact_token_shapes(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut output = String::with_capacity(input.len());
    let mut cursor = 0;

    while cursor < bytes.len() {
        // Only consider positions that start a token run, so `ghp_` inside a longer
        // word is not treated as a prefix.
        let at_run_start = cursor == 0 || !is_token_char(bytes[cursor - 1] as char);

        let matched = at_run_start
            .then(|| {
                SECRET_PREFIXES
                    .iter()
                    .find(|prefix| input[cursor..].starts_with(**prefix))
            })
            .flatten();

        if let Some(prefix) = matched {
            let end = cursor
                + input[cursor..]
                    .find(|character: char| !is_token_char(character))
                    .unwrap_or(input.len() - cursor);

            // Long enough to be a credential rather than a coincidence.
            if end - cursor >= prefix.len() + 12 {
                output.push_str(TOKEN_PLACEHOLDER);
                cursor = end;
                continue;
            }
        }

        let character = input[cursor..].chars().next().unwrap_or('\0');
        output.push(character);
        cursor += character.len_utf8();
    }

    output
}

/// `str::replace`, but matching without regard to ASCII case, and optionally only on
/// whole words.
fn replace_ignore_ascii_case(
    haystack: &str,
    needle: &str,
    replacement: &str,
    whole_word: bool,
) -> String {
    let lower_haystack = haystack.to_ascii_lowercase();
    let lower_needle = needle.to_ascii_lowercase();

    let mut output = String::with_capacity(haystack.len());
    let mut cursor = 0;

    while let Some(offset) = lower_haystack[cursor..].find(&lower_needle) {
        let start = cursor + offset;
        let end = start + needle.len();

        let bounded = !whole_word
            || (!haystack[..start].ends_with(is_token_char)
                && !haystack[end..].starts_with(is_token_char));

        output.push_str(&haystack[cursor..start]);

        if bounded {
            output.push_str(replacement);
        } else {
            output.push_str(&haystack[start..end]);
        }

        cursor = end;
    }

    output.push_str(&haystack[cursor..]);
    output
}

/// The real profile directory, asked of the shell rather than inferred from a path
/// pattern. Falls back to `%USERPROFILE%`, which Windows itself sets at logon.
#[cfg(windows)]
fn resolve_profile_dir() -> Option<PathBuf> {
    use windows::Win32::System::Com::CoTaskMemFree;
    use windows::Win32::UI::Shell::{FOLDERID_Profile, SHGetKnownFolderPath, KF_FLAG_DEFAULT};

    // SAFETY: on success this returns a COM-allocated, NUL-terminated wide string
    // that the caller owns; it is read and then freed with the shell allocator, and
    // not used afterwards.
    let from_shell = unsafe {
        SHGetKnownFolderPath(&FOLDERID_Profile, KF_FLAG_DEFAULT, None)
            .ok()
            .and_then(|wide| {
                let owned = wide.to_string().ok();
                CoTaskMemFree(Some(wide.as_ptr().cast()));
                owned
            })
    };

    from_shell
        .map(PathBuf::from)
        .or_else(fallback_profile_dir)
        .filter(|dir| !dir.as_os_str().is_empty())
}

#[cfg(not(windows))]
fn resolve_profile_dir() -> Option<PathBuf> {
    fallback_profile_dir()
}

/// Used when the shell API is unavailable, and on non-Windows so the golden test can
/// run anywhere.
fn fallback_profile_dir() -> Option<PathBuf> {
    std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .ok()
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_redactor() -> Redactor {
        Redactor::new(
            Some(PathBuf::from("C:\\Users\\alice")),
            Some("alice".to_owned()),
            Some("ALICE-LAPTOP".to_owned()),
        )
    }

    #[test]
    fn the_profile_path_is_replaced_before_the_bare_username() {
        let redactor = fixture_redactor();

        assert_eq!(
            redactor.redact("C:\\Users\\alice\\AppData\\Roaming\\io.github.valasme.wgm"),
            "%USERPROFILE%\\AppData\\Roaming\\io.github.valasme.wgm"
        );
    }

    #[test]
    fn a_bare_username_is_replaced_wherever_it_appears() {
        let redactor = fixture_redactor();

        assert_eq!(
            redactor.redact("user=alice logged in"),
            "user=%USERNAME% logged in"
        );
    }

    #[test]
    fn the_machine_name_is_replaced() {
        let redactor = fixture_redactor();

        assert_eq!(redactor.redact("host ALICE-LAPTOP"), "host %COMPUTERNAME%");
    }

    #[test]
    fn matching_ignores_case() {
        let redactor = fixture_redactor();

        assert_eq!(
            redactor.redact("c:\\users\\ALICE\\Desktop"),
            "%USERPROFILE%\\Desktop"
        );
    }

    #[test]
    fn forward_slashes_and_json_escaping_are_both_covered() {
        let redactor = fixture_redactor();

        assert_eq!(
            redactor.redact("C:/Users/alice/log.txt"),
            "%USERPROFILE%/log.txt"
        );
        assert_eq!(
            redactor.redact(r#"{"path":"C:\\Users\\alice\\settings.json"}"#),
            r#"{"path":"%USERPROFILE%\\settings.json"}"#
        );
    }

    /// The golden file from `docs/error-reporting.md` §9: a username, a profile path,
    /// a UNC home directory, a `D:\Users` path and a token-shaped string.
    #[test]
    fn the_golden_fixture_redacts_completely() {
        let input = include_str!("../../tests/fixtures/redaction/input.log");
        let expected = include_str!("../../tests/fixtures/redaction/expected.log");

        // The fixture describes a domain-joined machine with a redirected profile,
        // which is exactly the shape a `C:\Users\` regex would sail straight past.
        let redactor = Redactor::new(
            Some(PathBuf::from("\\\\fileserver\\home$\\dvalasellis")),
            Some("dvalasellis".to_owned()),
            Some("DEV-BOX-07".to_owned()),
        );

        assert_eq!(
            redactor.redact(input).replace("\r\n", "\n"),
            expected.replace("\r\n", "\n")
        );
    }

    #[test]
    fn a_redirected_profile_on_another_drive_is_still_redacted() {
        let redactor = Redactor::new(
            Some(PathBuf::from("D:\\Users\\bob")),
            Some("bob".to_owned()),
            None,
        );

        assert_eq!(
            redactor.redact("D:\\Users\\bob\\wgm.log"),
            "%USERPROFILE%\\wgm.log"
        );
    }

    #[test]
    fn an_unknown_username_is_reported_as_unsafe_rather_than_ignored() {
        let redactor = Redactor::new(None, None, None);

        assert!(
            !redactor.is_safe(),
            "a bundle must not be written without an identity"
        );
    }

    #[test]
    fn the_real_environment_yields_a_usable_redactor() {
        let redactor = Redactor::from_environment();

        // On any machine a test runs on, one of USERPROFILE or HOME is set.
        assert!(
            redactor.is_safe(),
            "could not resolve an identity to redact"
        );
    }

    #[test]
    fn token_shaped_strings_are_blanked() {
        let redactor = fixture_redactor();

        assert_eq!(
            redactor.redact("Authorization: Bearer ghp_A1b2C3d4E5f6G7h8I9j0K1l2M3n4"),
            "Authorization: Bearer %TOKEN%"
        );
        assert_eq!(redactor.redact("key=AKIAIOSFODNN7EXAMPLE1"), "key=%TOKEN%");
        assert_eq!(
            redactor.redact(r#"{"token":"github_pat_11ABCDEFG0abcdefg"}"#),
            r#"{"token":"%TOKEN%"}"#,
            "the surrounding punctuation must survive, or the file stops being JSON"
        );
    }

    #[test]
    fn a_short_prefix_lookalike_is_not_mistaken_for_a_credential() {
        let redactor = fixture_redactor();

        // Too short to be a token, and eating it would corrupt ordinary prose.
        assert_eq!(redactor.redact("sk-1"), "sk-1");
        // Not at the start of a run.
        assert_eq!(
            redactor.redact("xghp_A1b2C3d4E5f6G7h8"),
            "xghp_A1b2C3d4E5f6G7h8"
        );
    }

    /// The default Windows account is literally called `User`. Substring-matching it
    /// would rewrite `%USERPROFILE%` and every `user=` key in the log.
    #[test]
    fn a_generic_username_does_not_eat_the_rest_of_the_log() {
        let redactor = Redactor::new(
            Some(PathBuf::from("C:\\Users\\User")),
            Some("User".to_owned()),
            None,
        );

        assert_eq!(
            redactor.redact("wrote %USERPROFILE%\\settings.json for C:\\Users\\User"),
            "wrote %USERPROFILE%\\settings.json for %USERPROFILE%"
        );
        assert_eq!(
            redactor.redact("USERNAME resolution failed"),
            "USERNAME resolution failed"
        );
    }

    #[test]
    fn identifiers_a_maintainer_needs_survive_redaction() {
        let redactor = fixture_redactor();
        // A correlation id, a session id and a commit SHA are all "random-looking",
        // and a bundle with those mangled is a bundle nobody can act on.
        let line = "a3f9c1  session 4f2a91c0b7d3  commit 9c1e2a4b8d70";

        assert_eq!(redactor.redact(line), line);
    }

    #[test]
    fn redaction_does_not_corrupt_text_containing_no_identity() {
        let redactor = fixture_redactor();
        let untouched = "SETTINGS_WRITE_FAILED  path=\"%USERPROFILE%\\settings.json\"";

        assert_eq!(redactor.redact(untouched), untouched);
    }
}
