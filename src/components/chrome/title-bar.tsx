import { PanelLeftCloseIcon, PanelLeftIcon } from "lucide-react";

import { t } from "@/i18n/t";
import { cn } from "@/lib/cn";
import { useIsRail, useRailForced, useWorkspaceStore } from "@/stores/workspace-store";

import { WindowControls } from "./window-controls";
import { Wordmark } from "./wordmark";

/**
 * The Title Bar: 36px, replacing the native one.
 *
 * `data-tauri-drag-region` on the bar itself gives drag, double-click-to-maximise and
 * drag-to-unsnap for free; every interactive child opts out of it.
 *
 * DOM order is the focus order: sidebar toggle, then Window Controls, then the Sidebar
 * navigation below. Two tab stops before the nav is a smaller cost than a disordered
 * focus ring, and the skip link makes it one press to bypass both.
 */
export function TitleBar() {
  const collapsed = useIsRail();
  const railForced = useRailForced();
  const setSidebarCollapsed = useWorkspaceStore((state) => state.setSidebarCollapsed);

  const Icon = collapsed ? PanelLeftIcon : PanelLeftCloseIcon;

  return (
    <header
      data-tauri-drag-region
      className={cn(
        "flex h-[var(--title-bar-height)] shrink-0 items-stretch",
        "border-b border-border bg-bg",
        // The Title Bar never scrolls.
        "select-none",
      )}
    >
      {/* Rail-wide and centred, so the toggle sits on the same vertical axis as the
          Rail's icons directly below it. */}
      <div
        data-tauri-drag-region={false}
        className="flex w-[var(--rail-width)] shrink-0 items-center justify-center"
      >
        <button
          type="button"
          // Below 900px the Sidebar is forced to the Rail, so the toggle would be a
          // control that does nothing — and wgm does not render those.
          disabled={railForced}
          aria-label={collapsed ? t("chrome.showSidebar") : t("chrome.hideSidebar")}
          aria-expanded={!collapsed}
          onClick={() => void setSidebarCollapsed(!collapsed)}
          className={cn(
            "flex size-8 items-center justify-center rounded-sm",
            "text-text-muted transition-colors duration-100",
            "hover:bg-surface hover:text-text",
            "disabled:pointer-events-none disabled:opacity-40",
          )}
        >
          <Icon className="size-4" aria-hidden="true" />
        </button>
      </div>

      {/*
        No double-click handler: `data-tauri-drag-region` already gives drag,
        double-click-to-maximise and drag-to-unsnap, and duplicating the last one in
        JavaScript would toggle twice.
      */}
      <div data-tauri-drag-region className="flex flex-1 items-center">
        <Wordmark />
      </div>

      <WindowControls />
    </header>
  );
}
