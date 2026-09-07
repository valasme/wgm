/**
 * Regenerate the Instrument Serif Italic wordmark subset.
 *
 * The wordmark is three glyphs — `w`, `g`, `m` — and nothing else in wgm is set in a
 * serif. Shipping the full face would be ~30 KB of glyphs that are never drawn, on an
 * app whose only font requirement is that it never touches a CDN.
 *
 * Run: pnpm subset-wordmark
 *
 * The output is checked in. This script exists so the subset can be regenerated when
 * the upstream font updates, and so it is obvious what was done to it — see
 * scripts/subset-wordmark.md for the licence obligation that comes with subsetting.
 */

import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import subsetFont from "subset-font";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");

const SOURCE = join(
  root,
  "node_modules",
  "@fontsource",
  "instrument-serif",
  "files",
  "instrument-serif-latin-400-italic.woff2",
);
const OUTPUT = join(root, "public", "fonts", "instrument-serif-italic-wgm.woff2");

/** Every character the wordmark can ever contain. */
const GLYPHS = "wgm";

const source = readFileSync(SOURCE);

const subset = await subsetFont(source, GLYPHS, { targetFormat: "woff2" });

writeFileSync(OUTPUT, subset);

process.stdout.write(
  `  ${GLYPHS.length} glyphs · ${source.length} bytes -> ${subset.length} bytes` +
    ` (${Math.round((1 - subset.length / source.length) * 100)}% smaller)\n` +
    `  wrote ${OUTPUT}\n`,
);
