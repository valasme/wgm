/**
 * Regenerate `THIRD-PARTY-LICENSES.md`.
 *
 * Three sources, in one file, because a user asking "what is in this thing" should not
 * have to know that wgm has a Rust half and a JavaScript half:
 *
 * 1. `cargo about` for the crates,
 * 2. `pnpm licenses` for the npm packages,
 * 3. `licenses/` for the bundled fonts — the OFL requires its text to travel with the
 *    font, and a subset counts as a Modified Version.
 *
 * Run by `.github/workflows/licenses.yml`, and runnable by hand:
 *   node scripts/generate-third-party-licenses.mjs
 */

import { execFileSync } from "node:child_process";
import { readdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import process from "node:process";

const OUTPUT = "THIRD-PARTY-LICENSES.md";

/**
 * A source that cannot be read is a fatal error, not a footnote.
 *
 * This used to return null and let the caller write "_… was unavailable_" into the
 * file. That is how wgm shipped a licence list with no crates in it at all: the
 * workflow's `cargo install cargo-about` was a no-op without `--features cli`, every
 * run silently took the fallback, and the note read as a deliberate choice rather
 * than a broken tool. Failing here means a broken generator breaks the build.
 */
function run(command, args, options = {}) {
  try {
    return execFileSync(command, args, {
      encoding: "utf8",
      shell: process.platform === "win32",
      // `cargo about --format json` embeds the full text of every licence it found,
      // which is a few megabytes. The default 1 MB buffer truncates it into ENOBUFS.
      maxBuffer: 64 * 1024 * 1024,
      ...options,
    });
  } catch (error) {
    process.stderr.write(`\n  ${command} ${args.join(" ")} failed:\n  ${error.message}\n`);
    process.exit(1);
  }
}

/** npm packages, grouped by licence, from pnpm's own resolver. */
function npmSection() {
  const raw = run("pnpm", ["licenses", "list", "--json", "--prod"]);
  const byLicense = JSON.parse(raw);
  const lines = [];

  for (const [license, packages] of Object.entries(byLicense).sort()) {
    lines.push(`### ${license}\n`);
    for (const entry of packages.sort((a, b) => a.name.localeCompare(b.name))) {
      const versions = Array.isArray(entry.versions)
        ? entry.versions.join(", ")
        : entry.version;
      lines.push(`- \`${entry.name}\` ${versions}`);
    }
    lines.push("");
  }

  return lines.join("\n");
}

/**
 * Rust crates, via cargo-about, configured by `src-tauri/about.toml` — which is also
 * what narrows this to the crates that reach a Windows binary.
 */
function cargoSection() {
  const raw = run("cargo", [
    "about",
    "generate",
    "--manifest-path",
    "src-tauri/Cargo.toml",
    "--format",
    "json",
  ]);

  const report = JSON.parse(raw);

  // `licenses` is one entry per distinct licence *text*, so MIT alone arrives as 124
  // of them. Group by licence id to get the same shape as the npm section: one
  // heading per licence, the crates under it.
  const byLicense = new Map();

  for (const license of report.licenses ?? []) {
    const name = license.name ?? license.id;
    const crates = byLicense.get(name) ?? new Set();
    for (const used of license.used_by ?? []) {
      crates.add(`${used.crate.name} ${used.crate.version}`);
    }
    byLicense.set(name, crates);
  }

  const lines = [];

  for (const [name, crates] of [...byLicense].sort((a, b) => a[0].localeCompare(b[0]))) {
    lines.push(`### ${name}\n`);
    for (const crate of [...crates].sort()) {
      const at = crate.lastIndexOf(" ");
      lines.push(`- \`${crate.slice(0, at)}\` ${crate.slice(at + 1)}`);
    }
    lines.push("");
  }

  return lines.join("\n");
}

/** The bundled fonts. The OFL is explicit that its text ships with the font. */
function fontSection() {
  const lines = [];

  for (const file of readdirSync("licenses").sort()) {
    lines.push(`### ${file}\n`);
    lines.push("```");
    lines.push(readFileSync(join("licenses", file), "utf8").trimEnd());
    lines.push("```\n");
  }

  return lines.join("\n");
}

const document = `# Third-party licences

wgm itself is [MIT licensed](LICENSE). This file lists everything it is built from and
everything it ships.

**Generated — do not edit by hand.** Run
\`node scripts/generate-third-party-licenses.mjs\`, or let
\`.github/workflows/licenses.yml\` open a pull request when the dependencies change.

Generated ${new Date().toISOString().slice(0, 10)}.

---

## Bundled fonts

These are distributed *inside* the application, so their licence text travels with them.
The Instrument Serif file is a three-glyph subset, which the SIL Open Font License
treats as a Modified Version — hence the different family name. See
[\`scripts/subset-wordmark.md\`](scripts/subset-wordmark.md).

${fontSection()}
---

## Rust crates

${cargoSection()}
---

## npm packages

${npmSection()}`;

writeFileSync(OUTPUT, document);
process.stdout.write(`  wrote ${OUTPUT}\n`);
