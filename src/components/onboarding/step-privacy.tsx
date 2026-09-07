import type { Ref } from "react";

import { SettingSwitch } from "@/components/settings/setting-switch";
import { t } from "@/i18n/t";
import { commands } from "@/ipc";
import { commit } from "@/lib/commit";
import { useSettingsStore } from "@/stores/settings-store";

/**
 * The zero-network promise, stated plainly, plus the one toggle it has.
 *
 * Said here rather than buried in Settings because it is the thing about wgm most worth
 * knowing, and because a promise nobody reads is a promise asserted rather than kept.
 */
export function StepPrivacy({ headingRef }: { headingRef: Ref<HTMLHeadingElement> }) {
  const privacy = useSettingsStore((state) => state.settings?.privacy);

  if (!privacy) {
    return null;
  }

  return (
    <div className="flex w-full max-w-[52ch] flex-col items-center gap-6 text-center">
      <div className="flex flex-col gap-2">
        <h1 ref={headingRef} className="text-xl font-medium text-text outline-none">
          {t("onboarding.privacy.title")}
        </h1>
        <p className="text-sm text-text-muted">{t("onboarding.privacy.body")}</p>
      </div>

      <div className="w-full text-left">
        <SettingSwitch
          id="privacy.releaseCheckEnabled"
          label={t("onboarding.privacy.releaseCheck")}
          description={t("onboarding.privacy.releaseCheck.description")}
          value={privacy.releaseCheckEnabled}
          optimistic={(settings, next) => ({
            ...settings,
            privacy: { ...settings.privacy, releaseCheckEnabled: next },
          })}
          commit={(next) => commit(commands.settingsSetReleaseCheckEnabled(next))}
        />
      </div>
    </div>
  );
}
