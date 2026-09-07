import { AlertTriangleIcon } from "lucide-react";
import type { ReactNode } from "react";

import { t } from "@/i18n/t";
import { commands, type DocumentStatus } from "@/ipc";
import { cn } from "@/lib/cn";
import { useSettingsStore } from "@/stores/settings-store";
import { useWorkspaceStore } from "@/stores/workspace-store";

import { Button } from "./ui/button";

/**
 * Persistent banners for the states a toast cannot carry.
 *
 * Three of them, and each exists because the alternative is worse:
 *
 * - **Ephemeral Mode** — the user needs to know that nothing they change will survive,
 *   for the whole session, not for four seconds.
 * - **A corrupt document** — the banner names the backup file, and the user may not be
 *   looking when it happens.
 * - **Repeated write failures** — instant apply means every toggle writes, and a file
 *   locked by an antivirus scanner would otherwise produce a wall of toasts. **One
 *   coalesced banner**, driven by the consecutive-failure count Rust keeps.
 */
export function StatusBanners() {
  const settingsStatus = useSettingsStore((state) => state.status);
  const workspaceStatus = useWorkspaceStore((state) => state.status);

  const banners = [
    ...bannersFor(settingsStatus, "settings"),
    ...bannersFor(workspaceStatus, "workspace"),
  ];

  if (banners.length === 0) {
    return null;
  }

  return (
    <div className="flex flex-col">
      {banners.map((banner) => (
        <Banner
          key={banner.key}
          title={banner.title}
          body={banner.body}
          action={banner.action}
        />
      ))}
    </div>
  );
}

interface BannerSpec {
  key: string;
  title: string;
  body: string;
  action?: ReactNode;
}

function bannersFor(status: DocumentStatus | null, document: string): BannerSpec[] {
  if (status === null) {
    return [];
  }

  const banners: BannerSpec[] = [];

  if (status.ephemeral) {
    banners.push({
      key: `${document}-ephemeral`,
      title: t("banner.ephemeral.title"),
      body: t("banner.ephemeral.body"),
    });
  }

  if (status.outcome.kind === "corrupt") {
    banners.push({
      key: `${document}-corrupt`,
      title: t("banner.corrupt.title"),
      body: t("banner.corrupt.body", { backup: status.outcome.backup }),
      action: (
        <Button size="small" onClick={() => void commands.openKnownFolder("data")}>
          {t("banner.corrupt.action")}
        </Button>
      ),
    });
  }

  if (status.outcome.kind === "schemaTooNew") {
    banners.push({
      key: `${document}-schema`,
      title: t("banner.schemaTooNew.title"),
      body: t("banner.schemaTooNew.body"),
    });
  }

  if (status.consecutiveWriteFailures > 0) {
    banners.push({
      key: `${document}-writes`,
      title: t("banner.writeFailing.title"),
      body: t("banner.writeFailing.body"),
    });
  }

  return banners;
}

function Banner({ title, body, action }: { title: string; body: string; action?: ReactNode }) {
  return (
    <div
      role="status"
      className={cn(
        "flex items-start gap-3 border-b border-border bg-surface px-6 py-3",
        "text-sm text-text",
      )}
    >
      <AlertTriangleIcon
        className="mt-0.5 size-4 shrink-0 text-text-muted"
        aria-hidden="true"
      />

      <div className="flex flex-1 flex-col gap-1">
        <span className="font-medium">{title}</span>
        <span className="text-text-muted">{body}</span>
      </div>

      {action}
    </div>
  );
}
