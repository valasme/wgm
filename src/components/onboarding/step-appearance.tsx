import type { ReactNode, Ref } from "react";

import { AppearancePreview } from "@/components/appearance-preview";
import { t } from "@/i18n/t";
import { type Accent, type ColorSchemePreference, commands, type Density } from "@/ipc";
import { cn } from "@/lib/cn";
import { commit } from "@/lib/commit";
import { useSettingsStore } from "@/stores/settings-store";
import {
  ACCENT_LABELS,
  ACCENTS,
  COLOR_SCHEME_LABELS,
  COLOR_SCHEMES,
  DENSITIES,
  DENSITY_LABELS,
} from "@/theme/accents";
import { resolveColorScheme } from "@/theme/apply-theme";

/**
 * Appearance, with a preview of every option rather than only of the one in effect.
 *
 * The choice still applies as you click — the app itself is the final preview — but a
 * scheme you have not picked yet is invisible otherwise, and eight Accent swatches
 * that all render the *current* Accent (which is what they did) say nothing at all.
 * Each thumbnail carries its own `data-theme`/`data-accent`/`data-density`, so it is
 * painted by the same tokens the real chrome uses.
 */
export function StepAppearance({ headingRef }: { headingRef: Ref<HTMLHeadingElement> }) {
  const appearance = useSettingsStore((state) => state.settings?.appearance);
  const apply = useSettingsStore((state) => state.apply);

  if (!appearance) {
    return null;
  }

  // Swatches and Accent previews are drawn in the scheme that is actually on screen,
  // so what the user is choosing between is what they will get.
  const scheme = resolveColorScheme(appearance.colorScheme);

  return (
    <div className="flex w-full max-w-[64ch] flex-col items-center gap-8 text-center">
      <div className="flex flex-col gap-2">
        <h1 ref={headingRef} className="text-xl font-medium text-text outline-none">
          {t("onboarding.appearance.title")}
        </h1>
        <p className="text-sm text-text-muted">{t("onboarding.appearance.body")}</p>
      </div>

      <Group legend={t("settings.appearance.colorScheme")}>
        <div className="grid w-full grid-cols-4 gap-2">
          {COLOR_SCHEMES.map((option) => (
            <OptionCard
              key={option}
              label={t(COLOR_SCHEME_LABELS[option])}
              selected={appearance.colorScheme === option}
              onClick={() =>
                void apply(
                  "appearance.colorScheme",
                  (settings) => ({
                    ...settings,
                    appearance: { ...settings.appearance, colorScheme: option },
                  }),
                  () =>
                    commit(commands.settingsSetColorScheme(option as ColorSchemePreference)),
                )
              }
            >
              <AppearancePreview
                colorScheme={option}
                accent={appearance.accent}
                density={appearance.density}
              />
            </OptionCard>
          ))}
        </div>
      </Group>

      <Group legend={t("settings.appearance.accent")}>
        <div className="flex flex-wrap justify-center gap-3">
          {ACCENTS.map((option) => {
            const selected = appearance.accent === option;

            return (
              <button
                key={option}
                type="button"
                // The label is the accessible name; the swatch is the visual. Nothing is
                // signalled by colour alone — the selected one also carries a ring.
                aria-label={t(ACCENT_LABELS[option])}
                aria-pressed={selected}
                // Both attributes, not just the Accent: `--accent` is defined per scheme,
                // so a swatch that carries no scheme would be painted in the Light values
                // while the app around it is Dark.
                data-theme={scheme}
                data-accent={option}
                onClick={() =>
                  void apply(
                    "appearance.accent",
                    (settings) => ({
                      ...settings,
                      appearance: { ...settings.appearance, accent: option as Accent },
                    }),
                    () => commit(commands.settingsSetAccent(option as Accent)),
                  )
                }
                className={cn(
                  "size-7 rounded-full bg-accent transition-shadow duration-100",
                  "ring-offset-2 ring-offset-bg",
                  selected ? "ring-2 ring-text" : "ring-1 ring-border hover:ring-text-muted",
                )}
              />
            );
          })}
        </div>
      </Group>

      <Group legend={t("settings.appearance.density")}>
        <div className="grid w-full max-w-[32ch] grid-cols-2 gap-2">
          {DENSITIES.map((option) => (
            <OptionCard
              key={option}
              label={t(DENSITY_LABELS[option])}
              selected={appearance.density === option}
              onClick={() =>
                void apply(
                  "appearance.density",
                  (settings) => ({
                    ...settings,
                    appearance: { ...settings.appearance, density: option as Density },
                  }),
                  () => commit(commands.settingsSetDensity(option as Density)),
                )
              }
            >
              <AppearancePreview
                colorScheme={appearance.colorScheme}
                accent={appearance.accent}
                density={option}
              />
            </OptionCard>
          ))}
        </div>
      </Group>
    </div>
  );
}

/** A labelled group of options. The legend is visible: three unlabelled rows of
 * controls is the layout that made this step read as a puzzle. */
function Group({ legend, children }: { legend: string; children: ReactNode }) {
  return (
    <fieldset className="flex w-full flex-col items-center gap-2">
      <legend className="mb-2 text-xs text-text-muted">{legend}</legend>
      {children}
    </fieldset>
  );
}

/**
 * One option: its preview, its name, and an unmistakable selected state.
 *
 * Selected is an Accent border *and* a ring *and* a 500 weight — a single 1px border
 * change is the thing that was too quiet to see.
 */
function OptionCard({
  label,
  selected,
  onClick,
  children,
}: {
  label: string;
  selected: boolean;
  onClick: () => void;
  children: ReactNode;
}) {
  return (
    <button
      type="button"
      aria-pressed={selected}
      onClick={onClick}
      className={cn(
        "flex flex-col gap-2 rounded-md border p-2 transition-colors duration-100",
        selected
          ? "border-accent bg-surface font-medium text-text ring-1 ring-accent"
          : "border-border text-text-muted hover:border-text-muted hover:text-text",
      )}
    >
      {children}
      <span className="text-sm">{label}</span>
    </button>
  );
}
