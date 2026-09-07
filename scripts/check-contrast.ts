/**
 * Contrast gate.
 *
 * axe-core in jsdom cannot check colour contrast — jsdom computes no layout, so the
 * rule silently no-ops and the run comes back green. This script is the only thing in
 * the project that verifies the palette, so it checks the *whole* palette: the eight
 * Accent pairs and the neutral text ramp, in all three Color Schemes.
 *
 * It parses `src/theme/tokens.css` rather than importing a TypeScript copy of the
 * values, so there is exactly one source of truth and no way for the two to drift.
 *
 * Run: pnpm check-contrast
 */

import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const TOKENS_PATH = join(
  dirname(fileURLToPath(import.meta.url)),
  "..",
  "src",
  "theme",
  "tokens.css",
);

const SCHEMES = ["light", "dark", "darker"] as const;
const ACCENTS = [
  "blue",
  "violet",
  "cyan",
  "emerald",
  "amber",
  "orange",
  "rose",
  "neutral",
] as const;

type Scheme = (typeof SCHEMES)[number];
type Accent = (typeof ACCENTS)[number];

/** WCAG AA for normal-size text. `--text-muted` is normal-size text. */
const AA_TEXT = 4.5;
/** WCAG 1.4.11 for a non-text control. The focus ring is one. */
const AA_NON_TEXT = 3;

// ---------------------------------------------------------------- colour ---
// OKLCH -> Oklab -> linear sRGB -> sRGB, then WCAG relative luminance. Implemented
// here rather than pulled from a package: it is forty lines, it runs in CI, and a
// colour-space dependency is a supply-chain surface for no gain.

interface Oklch {
  l: number;
  c: number;
  h: number;
}

function parseOklch(input: string): Oklch {
  const match = input
    .trim()
    .match(/^oklch\(\s*([\d.]+)%?\s+([\d.]+)\s+([\d.]+)(?:deg)?\s*\)$/i);
  if (!match) {
    throw new Error(`not an oklch() colour: ${input}`);
  }
  const [, rawL, rawC, rawH] = match as unknown as [string, string, string, string];
  // `oklch(20% ...)` and `oklch(0.2 ...)` mean the same lightness.
  const l = input.includes("%") ? Number(rawL) / 100 : Number(rawL);
  return { l, c: Number(rawC), h: Number(rawH) };
}

function oklchToLinearSrgb({ l, c, h }: Oklch): [number, number, number] {
  const hRad = (h * Math.PI) / 180;
  const a = c * Math.cos(hRad);
  const b = c * Math.sin(hRad);

  const lCone = (l + 0.3963377774 * a + 0.2158037573 * b) ** 3;
  const mCone = (l - 0.1055613458 * a - 0.0638541728 * b) ** 3;
  const sCone = (l - 0.0894841775 * a - 1.291485548 * b) ** 3;

  return [
    4.0767416621 * lCone - 3.3077115913 * mCone + 0.2309699292 * sCone,
    -1.2684380046 * lCone + 2.6097574011 * mCone - 0.3413193965 * sCone,
    -0.0041960863 * lCone - 0.7034186147 * mCone + 1.707614701 * sCone,
  ];
}

/** Gamut-clip, then re-linearise. A channel outside [0,1] is what the screen shows. */
function clampChannel(value: number): number {
  return Math.min(1, Math.max(0, value));
}

function relativeLuminance(colour: Oklch): number {
  const [r, g, b] = oklchToLinearSrgb(colour).map(clampChannel) as [number, number, number];
  // The linear values above are already the linear-light sRGB the WCAG formula wants.
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

function contrastRatio(a: Oklch, b: Oklch): number {
  const la = relativeLuminance(a);
  const lb = relativeLuminance(b);
  const lighter = Math.max(la, lb);
  const darker = Math.min(la, lb);
  return (lighter + 0.05) / (darker + 0.05);
}

// ------------------------------------------------------------ token parse ---
// A deliberately small CSS reader: it finds `--name: value;` declarations inside the
// selector blocks we care about, in source order, so later blocks override earlier
// ones exactly as the cascade would.

interface Block {
  selector: string;
  declarations: Map<string, string>;
}

function parseBlocks(css: string): Block[] {
  const withoutComments = css.replace(/\/\*[\s\S]*?\*\//g, "");
  const blocks: Block[] = [];

  // Skip at-rules entirely: `forced-colors` yields to the system and
  // `prefers-reduced-motion` carries no colour.
  const body = withoutComments.replace(/@media[^{]*\{(?:[^{}]*\{[^{}]*\})*[^{}]*\}/g, "");

  const blockPattern = /([^{}]+)\{([^{}]*)\}/g;
  let match = blockPattern.exec(body);
  while (match !== null) {
    const selector = (match[1] ?? "").trim();
    const declarations = new Map<string, string>();
    for (const line of (match[2] ?? "").split(";")) {
      const colon = line.indexOf(":");
      if (colon === -1) continue;
      const name = line.slice(0, colon).trim();
      if (!name.startsWith("--")) continue;
      declarations.set(name, line.slice(colon + 1).trim());
    }
    blocks.push({ selector, declarations });
    match = blockPattern.exec(body);
  }

  return blocks;
}

/**
 * Does this selector apply to `<html data-theme=scheme data-accent=accent>`?
 * Only the handful of shapes tokens.css actually uses are understood; anything else
 * throws, so a new selector shape cannot slip past the gate unnoticed.
 */
function selectorApplies(selector: string, scheme: Scheme, accent: Accent): boolean {
  return selector
    .split(",")
    .map((part) => part.trim())
    .some((part) => {
      if (part === ":root") {
        // A bare `:root` is the Light default and the default Accent.
        return scheme === "light" && accent === "blue";
      }
      const attributes = [...part.matchAll(/\[data-(theme|accent)="([a-z]+)"\]/g)];
      const stripped = part.replace(/\[data-(theme|accent)="[a-z]+"\]/g, "").trim();
      // `[data-theme="dark"]` with no `:root` in front is the same rule applied to any
      // element carrying the attribute — which is what lets a preview panel render a
      // scheme the app is not in. It still applies to `<html>`, so the gate is unchanged.
      if (stripped !== ":root" && stripped !== "") {
        throw new Error(`check-contrast does not understand the selector: ${part}`);
      }
      return attributes.every(([, kind, value]) =>
        kind === "theme" ? value === scheme : value === accent,
      );
    });
}

const COLOUR_TOKENS = [
  "--bg",
  "--surface",
  "--border",
  "--text",
  "--text-muted",
  "--accent",
  "--accent-foreground",
];

/** Blocks that declare no colour cannot affect a contrast pair, so they are not
 * parsed for selector shape either — density and motion live in this file too. */
function declaresColour(block: Block): boolean {
  return COLOUR_TOKENS.some((token) => block.declarations.has(token));
}

function resolve(blocks: Block[], scheme: Scheme, accent: Accent): Map<string, string> {
  const resolved = new Map<string, string>();
  for (const block of blocks.filter(declaresColour)) {
    if (!selectorApplies(block.selector, scheme, accent)) continue;
    for (const [name, value] of block.declarations) {
      resolved.set(name, value);
    }
  }
  return resolved;
}

// ------------------------------------------------------------------ check ---

interface Result {
  scheme: Scheme;
  accent: Accent;
  label: string;
  ratio: number;
  required: number;
}

function require(
  tokens: Map<string, string>,
  name: string,
  scheme: Scheme,
  accent: Accent,
): Oklch {
  const raw = tokens.get(name);
  if (raw === undefined) {
    throw new Error(`${name} is not defined for theme=${scheme} accent=${accent}`);
  }
  return parseOklch(raw);
}

function main(): void {
  const blocks = parseBlocks(readFileSync(TOKENS_PATH, "utf8"));
  const results: Result[] = [];

  for (const scheme of SCHEMES) {
    for (const accent of ACCENTS) {
      const tokens = resolve(blocks, scheme, accent);
      const bg = require(tokens, "--bg", scheme, accent);
      const surface = require(tokens, "--surface", scheme, accent);
      const text = require(tokens, "--text", scheme, accent);
      const muted = require(tokens, "--text-muted", scheme, accent);
      const border = require(tokens, "--border", scheme, accent);
      const accentColour = require(tokens, "--accent", scheme, accent);
      const accentForeground = require(tokens, "--accent-foreground", scheme, accent);

      const push = (label: string, a: Oklch, b: Oklch, required: number) =>
        results.push({ scheme, accent, label, ratio: contrastRatio(a, b), required });

      // The neutral ramp is most of the pixels. Gating only the accents would leave
      // it unverified, which is the failure this script exists to prevent.
      push("--text on --bg", text, bg, AA_TEXT);
      push("--text on --surface", text, surface, AA_TEXT);
      push("--text-muted on --bg", muted, bg, AA_TEXT);
      push("--text-muted on --surface", muted, surface, AA_TEXT);

      // The primary button's label.
      push("--accent-foreground on --accent", accentForeground, accentColour, AA_TEXT);
      // The focus ring, which is a non-text control and must read against both
      // grounds it can be drawn over.
      push("--accent on --bg", accentColour, bg, AA_NON_TEXT);
      push("--accent on --surface", accentColour, surface, AA_NON_TEXT);
      // A hairline divider is the entire elevation model; if it vanishes the layout
      // does. 1.4.11 does not cover decorative borders, so this is advisory-strength
      // on purpose and set well below the control threshold.
      push("--border on --bg", border, bg, 1.2);
    }
  }

  // Accent-independent rows are identical across all eight accents; report each once.
  const seen = new Set<string>();
  const deduped = results.filter((result) => {
    const accentDependent = result.label.includes("--accent");
    const key = `${result.scheme}|${result.label}|${accentDependent ? result.accent : ""}`;
    if (seen.has(key)) return false;
    seen.add(key);
    return true;
  });

  const failures = deduped.filter((result) => result.ratio < result.required);

  for (const scheme of SCHEMES) {
    process.stdout.write(`\n  ${scheme}\n`);
    for (const result of deduped.filter((r) => r.scheme === scheme)) {
      const pass = result.ratio >= result.required;
      const accentDependent = result.label.includes("--accent");
      const name = accentDependent ? `${result.accent} · ${result.label}` : result.label;
      process.stdout.write(
        `    ${pass ? "ok  " : "FAIL"}  ${result.ratio.toFixed(2).padStart(6)} : 1` +
          `  (needs ${result.required})  ${name}\n`,
      );
    }
  }

  process.stdout.write(
    `\n  ${deduped.length - failures.length}/${deduped.length} pairs pass.\n`,
  );

  if (failures.length > 0) {
    process.stdout.write(
      `\n  ${failures.length} contrast failure(s). Adjust src/theme/tokens.css and` +
        " docs/design.md §2 together — they are meant to agree.\n",
    );
    process.exit(1);
  }
}

main();
