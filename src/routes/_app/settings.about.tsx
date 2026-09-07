import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useEffect, useState } from "react";
import { toast } from "sonner";

import { SettingRow } from "@/components/settings/setting-row";
import { SettingSection } from "@/components/settings/setting-section";
import { ShortcutSheet } from "@/components/shortcut-sheet";
import { Button } from "@/components/ui/button";
import type { MessageKey } from "@/i18n/t";
import { t } from "@/i18n/t";
import { type AppInfo, commands, type ReleaseCheck } from "@/ipc";
import { collectClientEnvironment } from "@/lib/environment";
import { useHeadingFocus } from "@/lib/focus";
import { messageFor, useSettingsStore } from "@/stores/settings-store";
import { useWorkspaceStore } from "@/stores/workspace-store";

export const Route = createFileRoute("/_app/settings/about")({
  component: AboutPage,
});

function AboutPage() {
  const headingRef = useHeadingFocus<HTMLHeadingElement>();
  const [info, setInfo] = useState<AppInfo | null>(null);
  const [release, setRelease] = useState<string | null>(null);
  const [releaseUrl, setReleaseUrl] = useState<string | null>(null);
  const [checking, setChecking] = useState(false);
  const [shortcutsOpen, setShortcutsOpen] = useState(false);

  const releaseCheckEnabled = useSettingsStore(
    (state) => state.settings?.privacy.releaseCheckEnabled ?? true,
  );
  const replayOnboarding = useWorkspaceStore((state) => state.replayOnboarding);
  const navigate = useNavigate();

  useEffect(() => {
    void commands.appInfo().then(setInfo);
  }, []);

  if (!info) {
    return null;
  }

  const checkForRelease = async () => {
    setChecking(true);
    setReleaseUrl(null);

    const result = await commands.checkForRelease();

    setChecking(false);
    setRelease(result.status === "ok" ? describe(result.data) : messageFor(result.error));

    if (result.status === "ok" && result.data.status === "newer") {
      setReleaseUrl(result.data.url);
    }
  };

  const copySummary = async () => {
    const summary = await commands.diagnosticsSummary(collectClientEnvironment(), {
      correlationId: null,
      errorCode: null,
    });
    await writeText(summary);
    toast.success(t("diagnostics.copySummary.done"));
  };

  return (
    <SettingSection heading={t("settings.about.title")} headingRef={headingRef}>
      <Fact label="settings.about.version" value={info.version} />
      <Fact label="settings.about.commit" value={info.commit} />
      <Fact label="settings.about.buildDate" value={info.buildDate} />
      <Fact label="settings.about.tauri" value={info.tauriVersion} />
      <Fact label="settings.about.webview" value={collectClientEnvironment().webviewVersion} />

      <SettingRow
        id="about.dataDirectory"
        // Already redacted by Rust: About is a screen people screenshot.
        label={t("settings.about.dataDirectory")}
        description={`${info.dataDir} · ${t(
          `settings.about.storageMode.${info.storageMode}` as MessageKey,
        )}`}
        control={() => (
          <Button onClick={() => void commands.openKnownFolder("data")}>
            {t("settings.about.openDataFolder")}
          </Button>
        )}
      />

      {info.pathNotes.map((note) => (
        <p key={note} className="max-w-[var(--content-max-width)] py-2 text-xs text-text-muted">
          {t(`pathNote.${note}` as MessageKey)}
        </p>
      ))}

      {releaseCheckEnabled && (
        <SettingRow
          id="about.checkForRelease"
          label={t("settings.about.checkForRelease")}
          description={release ?? undefined}
          control={() => (
            <div className="flex gap-2">
              {releaseUrl && (
                <Button variant="primary" onClick={() => void openUrl(releaseUrl)}>
                  {t("release.open")}
                </Button>
              )}
              <Button disabled={checking} onClick={() => void checkForRelease()}>
                {checking ? t("release.checking") : t("settings.about.checkForRelease")}
              </Button>
            </div>
          )}
        />
      )}

      <SettingRow
        id="about.copySummary"
        label={t("diagnostics.copySummary")}
        control={() => (
          <Button onClick={() => void copySummary()}>{t("diagnostics.copySummary")}</Button>
        )}
      />

      {/* The shortcut sheet is reachable by pointer here as well as by Ctrl+/ — a
          keyboard-only route to the keyboard shortcuts helps nobody who does not
          already know it. */}
      <SettingRow
        id="about.shortcuts"
        label={t("settings.about.shortcuts")}
        control={() => (
          <Button onClick={() => setShortcutsOpen(true)}>
            {t("settings.about.shortcuts")}
          </Button>
        )}
      />

      <SettingRow
        id="about.showSetupAgain"
        label={t("settings.about.showSetupAgain")}
        control={() => (
          <Button
            onClick={() => {
              void replayOnboarding().then(() => navigate({ to: "/onboarding" }));
            }}
          >
            {t("settings.about.showSetupAgain")}
          </Button>
        )}
      />

      <p className="max-w-[var(--content-max-width)] pt-4 text-xs text-text-muted">
        {t("settings.about.disclaimer")}
      </p>

      <ShortcutSheet open={shortcutsOpen} onOpenChange={setShortcutsOpen} />
    </SettingSection>
  );
}

function Fact({ label, value }: { label: MessageKey; value: string }) {
  return (
    <div className="flex max-w-[var(--content-max-width)] items-center justify-between gap-6 py-2">
      <span className="text-sm text-text">{t(label)}</span>
      <code data-selectable className="text-xs text-text-muted">
        {value}
      </code>
    </div>
  );
}

function describe(check: ReleaseCheck): string {
  return check.status === "newer"
    ? t("release.newer", { version: check.version })
    : t("release.upToDate");
}
