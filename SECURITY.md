# Security Policy

## Reporting a vulnerability

Report privately through **GitHub Security Advisories**:
<https://github.com/valasme/wgm/security/advisories/new>

Do not open a public issue for a security problem.

Please include what you can of: affected version, the steps to reproduce, what an
attacker gains, and whether it needs local access. A proof of concept helps but is
not required.

Expect an acknowledgement within 7 days and an assessment within 30. There is no
bounty programme; wgm is a single-maintainer project.

## There is no auto-update

This matters more here than it does in most projects. wgm never checks for, downloads
or installs a new version of itself on its own — see
[`docs/adr/0003-zero-network-policy.md`](docs/adr/0003-zero-network-policy.md).

**A security fix reaches you only when you install it.** Watch the repository for
releases (Watch → Custom → Releases) so you hear about one. Settings → About has a
*Check for a new version* button that queries the GitHub Releases API when you click
it, and does nothing else.

## Supported versions

The latest release only. There are no backported fixes to older versions.

## Scope

In scope:

- Anything that lets code outside wgm read or write wgm's data directory contents
  through wgm.
- Anything that defeats the redaction applied to a Diagnostics Bundle — that bundle
  is designed to be attached to a public issue, so a leak of a username, path or
  token in one is a real vulnerability.
- Anything that widens the webview's reach beyond the Content-Security-Policy and
  the capability allowlist in `src-tauri/capabilities/default.json`.
- Escalation of privilege during installation.

Out of scope:

- Vulnerabilities in winget, WebView2 or Windows itself. Report those to Microsoft.
- The SmartScreen warning shown for unsigned installers. It is expected and documented
  in the README; the mitigation is the published `SHA256SUMS.txt`.
- Findings that require an attacker who already has code execution as the same user.

## Verifying a download

Every release publishes `SHA256SUMS.txt`. Verify before installing:

```powershell
Get-FileHash .\wgm_0.1.0_x64-setup.exe -Algorithm SHA256
```

Compare the result against the matching line in `SHA256SUMS.txt`. Releases are not
code-signed yet, so this checksum is currently the whole integrity story.
