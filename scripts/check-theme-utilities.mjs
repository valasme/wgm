/**
 * Assert that the semantic colour utilities actually reach the built CSS.
 *
 * This exists because of a bug that every other gate passed straight over. In
 * `src/styles/global.css`, `--color-*: initial` clears the colour namespace *at the
 * point it appears* — so with the palette declared above it, the whole thing was
 * deleted and `bg-surface`, `text-text`, `border-border` and the rest generated
 * nothing at all. Tailwind does not warn: an unknown utility is simply not emitted.
 *
 * Nothing else could catch it. Biome and tsc see valid class strings, the tests render
 * a DOM with no stylesheet attached, and `check-contrast.ts` reads `tokens.css` rather
 * than the output. The only place the absence is visible is the build product, so that
 * is what this reads.
 *
 * Run after `pnpm build`:
 *   node scripts/check-theme-utilities.mjs
 */

import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import process from "node:process";

const ASSETS = join("dist", "assets");

/** One per colour token in the theme, plus the shapes they are used through. */
const REQUIRED = [
  "bg-bg",
  "bg-surface",
  "bg-accent",
  "text-text",
  "text-text-muted",
  "text-accent-foreground",
  "border-border",
  "divide-border",
  "hover:bg-surface",
];

let css;
try {
  const sheets = readdirSync(ASSETS).filter((file) => file.endsWith(".css"));
  if (sheets.length === 0) {
    throw new Error("no stylesheet");
  }
  css = sheets.map((file) => readFileSync(join(ASSETS, file), "utf8")).join("\n");
} catch {
  process.stderr.write(`  no built CSS in ${ASSETS}. Run \`pnpm build\` first.\n`);
  process.exit(1);
}

// Tailwind escapes `:` in a class name, so `hover:bg-surface` is `.hover\:bg-surface`.
const missing = REQUIRED.filter((utility) => !css.includes(utility.replace(":", "\\:")));

if (missing.length > 0) {
  process.stderr.write(
    `\n  ${missing.length} theme utility/utilities generated nothing:\n` +
      missing.map((utility) => `    ${utility}\n`).join("") +
      "\n  The colour namespace is probably being cleared after it is populated." +
      "\n  In src/styles/global.css, `--color-*: initial` must come *before* the" +
      "\n  @theme inline block that defines the palette.\n\n",
  );
  process.exit(1);
}

process.stdout.write(`  ${REQUIRED.length}/${REQUIRED.length} theme utilities generated.\n`);
