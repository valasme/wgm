import { createFileRoute, Outlet, redirect, useNavigate } from "@tanstack/react-router";
import { useCallback, useState } from "react";
import { Sidebar } from "@/components/chrome/sidebar";
import { TitleBar } from "@/components/chrome/title-bar";
import { CommandPalette } from "@/components/command-palette";
import { RouteError } from "@/components/errors/route-error";
import { RouteAnnouncer } from "@/components/route-announcer";
import { ShortcutSheet } from "@/components/shortcut-sheet";
import { StatusBanners } from "@/components/status-banners";
import { t } from "@/i18n/t";
import { commands } from "@/ipc";
import { type NAV_SHORTCUT_TARGETS, useShortcuts } from "@/lib/shortcuts";
import { useZoom } from "@/lib/zoom";
import { useIsRail, useWorkspaceStore } from "@/stores/workspace-store";

export const Route = createFileRoute("/_app")({
  component: AppShell,
  errorComponent: RouteError,
  beforeLoad: () => {
    // First run goes to setup. Progress lives in Workspace State, which is loaded
    // before the first render — and which is never imported, so someone else's
    // settings file cannot skip your setup.
    const onboarding = useWorkspaceStore.getState().workspace?.onboarding;

    if (onboarding && onboarding.completedVersion === null) {
      throw redirect({ to: "/onboarding" });
    }
  },
});

/**
 * Title Bar + Sidebar + Content Area.
 *
 * Landmark structure per docs/accessibility.md §2: `<header>` for the Title Bar,
 * `<nav aria-label>` for the Sidebar, and `<main id="main" tabindex="-1">` for the
 * Content Area. The Content Area scrolls; the Title Bar and Sidebar never do.
 */
function AppShell() {
  const navigate = useNavigate();
  const [paletteOpen, setPaletteOpen] = useState(false);
  const [shortcutsOpen, setShortcutsOpen] = useState(false);

  // Below 900px the Sidebar forces to the Rail. Read during render rather than
  // measured in an effect, so the shell never commits an expanded frame and then
  // animates it closed. It is view state either way and is never written to Workspace
  // State — a narrow window must not permanently collapse the Sidebar for the next
  // launch.
  const collapsed = useIsRail();
  const setSidebarCollapsed = useWorkspaceStore((state) => state.setSidebarCollapsed);

  // Ctrl +/-/0. Off by default in Tauri, and the cheapest low-vision accommodation
  // available in a window with no browser chrome to offer it.
  useZoom();

  useShortcuts({
    onPalette: useCallback(() => setPaletteOpen((open) => !open), []),
    onToggleSidebar: useCallback(
      () => void setSidebarCollapsed(!collapsed),
      [collapsed, setSidebarCollapsed],
    ),
    onSettings: useCallback(() => void navigate({ to: "/settings" }), [navigate]),
    onHelp: useCallback(() => setShortcutsOpen(true), []),
    onSystemMenu: useCallback(() => void commands.openSystemMenu(), []),
    onNavigate: useCallback(
      (to: (typeof NAV_SHORTCUT_TARGETS)[number]) => void navigate({ to }),
      [navigate],
    ),
  });

  return (
    <div className="flex h-full flex-col overflow-hidden">
      {/* First in the tab order, so both Title Bar stops can be bypassed in one press. */}
      <a href="#main" className="skip-link">
        {t("chrome.skipToContent")}
      </a>

      <TitleBar />

      <div className="flex flex-1 overflow-hidden">
        <Sidebar onOpenPalette={() => setPaletteOpen(true)} />

        <main
          id="main"
          tabIndex={-1}
          className="flex flex-1 flex-col overflow-y-auto outline-none"
        >
          <StatusBanners />
          <Outlet />
        </main>
      </div>

      <RouteAnnouncer />

      <CommandPalette open={paletteOpen} onOpenChange={setPaletteOpen} />
      <ShortcutSheet open={shortcutsOpen} onOpenChange={setShortcutsOpen} />
    </div>
  );
}
