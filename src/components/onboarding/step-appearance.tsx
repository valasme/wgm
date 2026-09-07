import type { Ref } from "react";

import { t } from "@/i18n/t";
import { type Accent, type ColorSchemePreference, commands } from "@/ipc";
import { cn } from "@/lib/cn";
import { commit } from "@/lib/commit";
import { useSettingsStore } from "@/stores/settings-store";
import { ACCENT_LABELS, ACCENTS, COLOR_SCHEME_LABELS, COLOR_SCHEMES } from "@/theme/accents";

/**
 * Appearance, with a live preview — the change is applied as you click, because the
 * preview *is* the application. A separate preview panel would be a second thing to
 * keep in step with the first.
 */
export function StepAppearance({ headingRef }: { headingRef: Ref<HTMLHeadingElement> }) {
  const appearance = useSettingsStore((state) => state.settings?.appearance);
  const apply = useSettingsStore((state) => state.apply);

  if (!appearance) {
    return null;
  }

  return (
    <div className="flex max-w-[52ch] flex-col items-center gap-6 text-center">
      <div className="flex flex-col gap-2">
        <h1 ref={headingRef} className="text-xl font-medium text-text outline-none">
          {t("onboarding.appearance.title")}
        </h1>
        <p className="text-sm text-text-muted">{t("onboarding.appearance.body")}</p>
      </div>

      <fieldset className="flex flex-col gap-2">
        <legend className="sr-only">{t("settings.appearance.colorScheme")}</legend>
        <div className="flex gap-2">
          {COLOR_SCHEMES.map((scheme) => (
            <button
              key={scheme}
              type="button"
              aria-pressed={appearance.colorScheme === scheme}
              onClick={() =>
                void apply(
                  "appearance.colorScheme",
                  (settings) => ({
                    ...settings,
                    appearance: { ...settings.appearance, colorScheme: scheme },
                  }),
                  () =>
                    commit(commands.settingsSetColorScheme(scheme as ColorSchemePreference)),
                )
              }
              className={cn(
                "h-8 rounded-sm border px-3 text-sm transition-colors duration-100",
                appearance.colorScheme === scheme
                  ? "border-accent bg-surface font-medium text-text"
                  : "border-border text-text-muted hover:text-text",
              )}
            >
              {t(COLOR_SCHEME_LABELS[scheme])}
            </button>
          ))}
        </div>
      </fieldset>

      <fieldset className="flex flex-col gap-2">
        <legend className="sr-only">{t("settings.appearance.accent")}</legend>
        <div className="flex flex-wrap justify-center gap-2">
          {ACCENTS.map((accent) => (
            <button
              key={accent}
              type="button"
              // The label is the accessible name; the swatch is the visual. Nothing is
              // signalled by colour alone — the selected one also carries a ring.
              aria-label={t(ACCENT_LABELS[accent])}
              aria-pressed={appearance.accent === accent}
              data-accent={accent}
              onClick={() =>
                void apply(
                  "appearance.accent",
                  (settings) => ({
                    ...settings,
                    appearance: { ...settings.appearance, accent: accent as Accent },
                  }),
                  () => commit(commands.settingsSetAccent(accent as Accent)),
                )
              }
              className={cn(
                "size-6 rounded-full border-2 bg-accent transition-colors duration-100",
                appearance.accent === accent ? "border-text" : "border-transparent",
              )}
            />
          ))}
        </div>
      </fieldset>
    </div>
  );
}
