import { createFileRoute } from "@tanstack/react-router";

import { SettingSection } from "@/components/settings/setting-section";
import { SettingSwitch } from "@/components/settings/setting-switch";
import { t } from "@/i18n/t";
import { commands } from "@/ipc";
import { commit } from "@/lib/commit";
import { useHeadingFocus } from "@/lib/focus";
import { useSettingsStore } from "@/stores/settings-store";

export const Route = createFileRoute("/_app/settings/general")({
  component: GeneralPage,
});

/**
 * General.
 *
 * **Confirm before destructive actions is rendered only because it genuinely gates
 * Reset to defaults** — it is the one exception to instant apply, and the only reason
 * a control this abstract earns a row at all. A language selector is deliberately
 * absent until a second catalog exists.
 */
function GeneralPage() {
  const headingRef = useHeadingFocus<HTMLHeadingElement>();
  const general = useSettingsStore((state) => state.settings?.general);

  if (!general) {
    return null;
  }

  return (
    <SettingSection heading={t("settings.general.title")} headingRef={headingRef}>
      <SettingSwitch
        id="general.launchOnStartup"
        label={t("settings.general.launchOnStartup")}
        description={t("settings.general.launchOnStartup.description")}
        value={general.launchOnStartup}
        optimistic={(settings, next) => ({
          ...settings,
          general: { ...settings.general, launchOnStartup: next },
        })}
        commit={(next) => commit(commands.settingsSetLaunchOnStartup(next))}
      />

      <SettingSwitch
        id="general.startMinimized"
        label={t("settings.general.startMinimized")}
        description={t("settings.general.startMinimized.description")}
        value={general.startMinimized}
        optimistic={(settings, next) => ({
          ...settings,
          general: { ...settings.general, startMinimized: next },
        })}
        commit={(next) => commit(commands.settingsSetStartMinimized(next))}
      />

      <SettingSwitch
        id="general.restoreWindowPosition"
        label={t("settings.general.restoreWindowPosition")}
        description={t("settings.general.restoreWindowPosition.description")}
        value={general.restoreWindowPosition}
        optimistic={(settings, next) => ({
          ...settings,
          general: { ...settings.general, restoreWindowPosition: next },
        })}
        commit={(next) => commit(commands.settingsSetRestoreWindowPosition(next))}
      />

      <SettingSwitch
        id="general.confirmDestructive"
        label={t("settings.general.confirmDestructive")}
        description={t("settings.general.confirmDestructive.description")}
        value={general.confirmDestructiveActions}
        optimistic={(settings, next) => ({
          ...settings,
          general: { ...settings.general, confirmDestructiveActions: next },
        })}
        commit={(next) => commit(commands.settingsSetConfirmDestructiveActions(next))}
      />
    </SettingSection>
  );
}
