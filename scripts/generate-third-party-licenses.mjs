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

function run(command, args, options = {}) {
  try {
    return execFileSync(command, args, {
      encoding: "utf8",
      shell: process.platform === "win32",
      ...options,
    });
  } catch (error) {
    process.stderr.write(`  ${command} failed: ${error.message}\n`);
    return null;
  }
}

/** npm packages, grouped by licence, from pnpm's own resolver. */
function npmSection() {
  const raw = run("pnpm", ["licenses", "list", "--json", "--prod"]);

  if (!raw) {
    return "_`pnpm licenses` was unavailable when this file was generated._\n";
  }

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

/** Rust crates, via cargo-about. Falls back to a plain list from `cargo metadata`. */
function cargoSection() {
  const raw = run("cargo", [
    "about",
    "generate",
    "--manifest-path",
    "src-tauri/Cargo.toml",
    "--format",
    "json",
  ]);

  if (!raw) {
    return "_`cargo about` was unavailable when this file was generated._\n";
  }

  const report = JSON.parse(raw);
  const lines = [];

  for (const license of report.licenses ?? []) {
    lines.push(`### ${license.name ?? license.id}\n`);
    for (const used of license.used_by ?? []) {
      lines.push(`- \`${used.crate.name}\` ${used.crate.version}`);
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
