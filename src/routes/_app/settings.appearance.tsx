import { createFileRoute } from "@tanstack/react-router";

import { SettingSection } from "@/components/settings/setting-section";
import { SettingSelect } from "@/components/settings/setting-select";
import { t } from "@/i18n/t";
import {
  type Accent,
  type ColorSchemePreference,
  commands,
  type Density,
  type ReduceMotion,
} from "@/ipc";
import { commit } from "@/lib/commit";
import { useHeadingFocus } from "@/lib/focus";
import { useSettingsStore } from "@/stores/settings-store";
import {
  ACCENT_LABELS,
  ACCENTS,
  COLOR_SCHEME_LABELS,
  COLOR_SCHEMES,
  DENSITIES,
  DENSITY_LABELS,
  REDUCE_MOTION_LABELS,
  REDUCE_MOTION_OPTIONS,
} from "@/theme/accents";

export const Route = createFileRoute("/_app/settings/appearance")({
  component: AppearancePage,
});

function AppearancePage() {
  const headingRef = useHeadingFocus<HTMLHeadingElement>();
  const appearance = useSettingsStore((state) => state.settings?.appearance);

  if (!appearance) {
    return null;
  }

  return (
    <SettingSection heading={t("settings.appearance.title")} headingRef={headingRef}>
      <SettingSelect<ColorSchemePreference>
        id="appearance.colorScheme"
        label={t("settings.appearance.colorScheme")}
        description={t("settings.appearance.colorScheme.description")}
        value={appearance.colorScheme}
        options={COLOR_SCHEMES.map((scheme) => ({
          value: scheme,
          label: t(COLOR_SCHEME_LABELS[scheme]),
        }))}
        optimistic={(settings, next) => ({
          ...settings,
          appearance: { ...settings.appearance, colorScheme: next },
        })}
        commit={(next) => commit(commands.settingsSetColorScheme(next))}
      />

      <SettingSelect<Accent>
        id="appearance.accent"
        label={t("settings.appearance.accent")}
        description={t("settings.appearance.accent.description")}
        value={appearance.accent}
        options={ACCENTS.map((accent) => ({ value: accent, label: t(ACCENT_LABELS[accent]) }))}
        optimistic={(settings, next) => ({
          ...settings,
          appearance: { ...settings.appearance, accent: next },
        })}
        commit={(next) => commit(commands.settingsSetAccent(next))}
      />

      <SettingSelect<Density>
        id="appearance.density"
        label={t("settings.appearance.density")}
        description={t("settings.appearance.density.description")}
        value={appearance.density}
        options={DENSITIES.map((density) => ({
          value: density,
          label: t(DENSITY_LABELS[density]),
        }))}
        optimistic={(settings, next) => ({
          ...settings,
          appearance: { ...settings.appearance, density: next },
        })}
        commit={(next) => commit(commands.settingsSetDensity(next))}
      />

      <SettingSelect<ReduceMotion>
        id="appearance.reduceMotion"
        label={t("settings.appearance.reduceMotion")}
        description={t("settings.appearance.reduceMotion.description")}
        value={appearance.reduceMotion}
        options={REDUCE_MOTION_OPTIONS.map((option) => ({
          value: option,
          label: t(REDUCE_MOTION_LABELS[option]),
        }))}
        optimistic={(settings, next) => ({
          ...settings,
          appearance: { ...settings.appearance, reduceMotion: next },
        })}
        commit={(next) => commit(commands.settingsSetReduceMotion(next))}
      />
    </SettingSection>
  );
}
