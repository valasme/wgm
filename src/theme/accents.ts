import type { MessageKey } from "@/i18n/t";
import type { Accent, ColorSchemePreference, Density, ReduceMotion } from "@/ipc";

/**
 * The eight Accents, in the order they are offered.
 *
 * The **colours** are not here: they live in `src/theme/tokens.css`, which
 * `scripts/check-contrast.ts` parses and gates. This file holds only the identity and
 * the label, so there is no second copy of a value nobody has verified.
 */
export const ACCENTS = [
  "blue",
  "violet",
  "cyan",
  "emerald",
  "amber",
  "orange",
  "rose",
  "neutral",
] as const satisfies readonly Accent[];

export const ACCENT_LABELS: Record<Accent, MessageKey> = {
  blue: "settings.appearance.accent.blue",
  violet: "settings.appearance.accent.violet",
  cyan: "settings.appearance.accent.cyan",
  emerald: "settings.appearance.accent.emerald",
  amber: "settings.appearance.accent.amber",
  orange: "settings.appearance.accent.orange",
  rose: "settings.appearance.accent.rose",
  neutral: "settings.appearance.accent.neutral",
};

/**
 * The Color Scheme preferences, in the order they are offered.
 *
 * `System` is a preference, not a scheme: it resolves to Light or Dark and **never**
 * to Darker, because the operating system has no signal for "extra dark" and nobody's
 * OS is asking for it. Darker is a deliberate opt-in.
 */
export const COLOR_SCHEMES = [
  "system",
  "light",
  "dark",
  "darker",
] as const satisfies readonly ColorSchemePreference[];

export const COLOR_SCHEME_LABELS: Record<ColorSchemePreference, MessageKey> = {
  system: "settings.appearance.colorScheme.system",
  light: "settings.appearance.colorScheme.light",
  dark: "settings.appearance.colorScheme.dark",
  darker: "settings.appearance.colorScheme.darker",
};

export const DENSITIES = ["comfortable", "compact"] as const satisfies readonly Density[];

export const DENSITY_LABELS: Record<Density, MessageKey> = {
  comfortable: "settings.appearance.density.comfortable",
  compact: "settings.appearance.density.compact",
};

export const REDUCE_MOTION_OPTIONS = [
  "system",
  "on",
  "off",
] as const satisfies readonly ReduceMotion[];

export const REDUCE_MOTION_LABELS: Record<ReduceMotion, MessageKey> = {
  system: "settings.appearance.reduceMotion.system",
  on: "settings.appearance.reduceMotion.on",
  off: "settings.appearance.reduceMotion.off",
};

/** The three schemes that can actually be in effect. `system` is not one of them. */
export type ResolvedColorScheme = "light" | "dark" | "darker";
