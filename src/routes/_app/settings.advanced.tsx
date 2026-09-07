import { createFileRoute } from "@tanstack/react-router";
import { open as openFile, save } from "@tauri-apps/plugin-dialog";
import { useState } from "react";
import { toast } from "sonner";
import { SectionBoundary } from "@/components/errors/section-error";
import { BundlePreviewDialog } from "@/components/settings/bundle-preview";
import { RecentProblems } from "@/components/settings/recent-problems";
import { SettingRow } from "@/components/settings/setting-row";
import { SettingSection } from "@/components/settings/setting-section";
import { SettingSelect } from "@/components/settings/setting-select";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { t } from "@/i18n/t";
import { commands, type ImportPreview, type LogLevel } from "@/ipc";
import { commit } from "@/lib/commit";
import { useHeadingFocus } from "@/lib/focus";
import { useSettingsStore } from "@/stores/settings-store";

export const Route = createFileRoute("/_app/settings/advanced")({
  component: AdvancedPage,
});

const LOG_LEVELS: { value: LogLevel; label: string }[] = [
  { value: "error", label: t("settings.advanced.logLevel.error") },
  { value: "warn", label: t("settings.advanced.logLevel.warn") },
  { value: "info", label: t("settings.advanced.logLevel.info") },
  { value: "debug", label: t("settings.advanced.logLevel.debug") },
  { value: "trace", label: t("settings.advanced.logLevel.trace") },
];

const RETENTION_DAYS = [1, 7, 14, 30, 90];

function AdvancedPage() {
  const headingRef = useHeadingFocus<HTMLHeadingElement>();
  const advanced = useSettingsStore((state) => state.settings?.advanced);
  const confirmDestructive = useSettingsStore(
    (state) => state.settings?.general.confirmDestructiveActions ?? true,
  );
  const load = useSettingsStore((state) => state.load);

  const [bundleOpen, setBundleOpen] = useState(false);
  const [resetOpen, setResetOpen] = useState(false);
  const [importPreview, setImportPreview] = useState<{
    path: string;
    preview: ImportPreview;
  } | null>(null);

  if (!advanced) {
    return null;
  }

  const doReset = async () => {
    const result = await commands.settingsReset();
    if (result.status === "ok") {
      await load();
      toast.success(t("reset.done"));
    }
    setResetOpen(false);
  };

  const onExportSettings = async () => {
    const path = await save({
      defaultPath: "wgm-settings.json",
      filters: [{ name: "JSON", extensions: ["json"] }],
    });

    if (!path) {
      return;
    }

    const result = await commands.settingsExport(path);
    if (result.status === "ok") {
      toast.success(t("export.done"));
    }
  };

  const onPickImport = async () => {
    const path = await openFile({
      multiple: false,
      filters: [{ name: "JSON", extensions: ["json"] }],
    });

    if (typeof path !== "string") {
      return;
    }

    const result = await commands.settingsImportPreview(path);
    if (result.status === "ok") {
      setImportPreview({ path, preview: result.data });
    }
  };

  const onApplyImport = async () => {
    if (!importPreview) {
      return;
    }

    const result = await commands.settingsImportApply(importPreview.path);
    if (result.status === "ok") {
      await load();
      toast.success(t("import.done"));
    }
    setImportPreview(null);
  };

  return (
    <SettingSection heading={t("settings.advanced.title")} headingRef={headingRef}>
      <SettingSelect<LogLevel>
        id="advanced.logLevel"
        label={t("settings.advanced.logLevel")}
        description={t("settings.advanced.logLevel.description")}
        value={advanced.logLevel}
        options={LOG_LEVELS}
        optimistic={(settings, next) => ({
          ...settings,
          advanced: { ...settings.advanced, logLevel: next },
        })}
        commit={(next) => commit(commands.settingsSetLogLevel(next))}
      />

      <SettingSelect<`${number}`>
        id="advanced.logRetentionDays"
        label={t("settings.advanced.logRetention")}
        description={t("settings.advanced.logRetention.description")}
        value={`${advanced.logRetentionDays}` as `${number}`}
        options={RETENTION_DAYS.map((days) => ({
          value: `${days}` as `${number}`,
          label: t("settings.advanced.logRetention.days", { days }),
        }))}
        optimistic={(settings, next) => ({
          ...settings,
          advanced: { ...settings.advanced, logRetentionDays: Number(next) },
        })}
        commit={(next) => commit(commands.settingsSetLogRetentionDays(Number(next)))}
      />

      <SettingRow
        id="advanced.openLogFolder"
        label={t("settings.advanced.openLogFolder")}
        control={() => (
          <Button onClick={() => void commands.openKnownFolder("logs")}>
            {t("settings.advanced.openLogFolder")}
          </Button>
        )}
      />

      <SettingRow
        id="advanced.exportDiagnostics"
        label={t("settings.advanced.exportDiagnostics")}
        description={t("settings.advanced.exportDiagnostics.description")}
        control={() => (
          <Button onClick={() => setBundleOpen(true)}>
            {t("settings.advanced.exportDiagnostics")}
          </Button>
        )}
      />

      <SettingRow
        id="advanced.importSettings"
        label={t("settings.advanced.importSettings")}
        control={() => (
          <Button onClick={() => void onPickImport()}>
            {t("settings.advanced.importSettings")}
          </Button>
        )}
      />

      <SettingRow
        id="advanced.exportSettings"
        label={t("settings.advanced.exportSettings")}
        control={() => (
          <Button onClick={() => void onExportSettings()}>
            {t("settings.advanced.exportSettings")}
          </Button>
        )}
      />

      <SettingRow
        id="advanced.reset"
        label={t("settings.advanced.reset")}
        description={t("settings.advanced.reset.description")}
        control={() => (
          <Button onClick={() => (confirmDestructive ? setResetOpen(true) : void doReset())}>
            {t("settings.advanced.reset")}
          </Button>
        )}
      />

      <div className="pt-4">
        <h3 className="mb-1 text-sm font-medium text-text">
          {t("settings.advanced.recentProblems")}
        </h3>
        {/* Its own boundary: reading the ring buffer can fail without that being a
            reason to lose the rest of Settings. */}
        <SectionBoundary name="recent-problems">
          <RecentProblems />
        </SectionBoundary>
      </div>

      <SectionBoundary name="bundle-preview">
        <BundlePreviewDialog open={bundleOpen} onOpenChange={setBundleOpen} />
      </SectionBoundary>

      {/* The one exception to instant apply, and the only reason
          confirm-before-destructive-actions is rendered at all. */}
      <Dialog open={resetOpen} onOpenChange={setResetOpen}>
        <DialogContent className="max-w-md">
          <DialogHeader>
            <DialogTitle>{t("reset.title")}</DialogTitle>
            <DialogDescription>{t("reset.body")}</DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button onClick={() => setResetOpen(false)}>{t("reset.cancel")}</Button>
            <Button variant="primary" onClick={() => void doReset()}>
              {t("reset.confirm")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* An older schema is migrated and then *previewed* — "this will change 4
          settings" — before anything is applied. */}
      <Dialog open={importPreview !== null} onOpenChange={() => setImportPreview(null)}>
        <DialogContent className="max-w-lg">
          <DialogHeader>
            <DialogTitle>{t("import.title")}</DialogTitle>
            <DialogDescription>
              {importPreview === null
                ? ""
                : importPreview.preview.changes.length === 0
                  ? t("import.noChanges")
                  : importPreview.preview.changes.length === 1
                    ? t("import.previewOne")
                    : t("import.preview", { count: importPreview.preview.changes.length })}
            </DialogDescription>
          </DialogHeader>

          <ul className="flex max-h-64 flex-col divide-y divide-border overflow-y-auto">
            {importPreview?.preview.changes.map((change) => (
              <li key={change.path} className="flex items-center justify-between gap-4 py-2">
                <code className="truncate text-xs text-text">{change.path}</code>
                <span className="shrink-0 text-xs text-text-muted">
                  {change.current} → {change.incoming}
                </span>
              </li>
            ))}
          </ul>

          <p className="text-xs text-text-muted">{t("import.backupNote")}</p>

          <DialogFooter>
            <Button onClick={() => setImportPreview(null)}>{t("import.cancel")}</Button>
            <Button variant="primary" onClick={() => void onApplyImport()}>
              {t("import.apply")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </SettingSection>
  );
}
