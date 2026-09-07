import { Link, useRouterState } from "@tanstack/react-router";
import {
  BoxIcon,
  HardDriveDownloadIcon,
  RefreshCwIcon,
  SearchIcon,
  SettingsIcon,
} from "lucide-react";
import type { ComponentType } from "react";
import type { MessageKey } from "@/i18n/t";
import { t } from "@/i18n/t";
import { cn } from "@/lib/cn";
import { formatShortcut, SHORTCUTS } from "@/lib/shortcuts";
import { isRail, useWorkspaceStore } from "@/stores/workspace-store";
import { Tooltip } from "../ui/tooltip";
import { SidebarResizer } from "./sidebar-resizer";

interface NavItem {
  to: string;
  label: MessageKey;
  icon: ComponentType<{ className?: string }>;
}

/** Search · Kits · Installed · Updates, with Settings pinned to the bottom. */
const PRIMARY: NavItem[] = [
  { to: "/search", label: "nav.search", icon: SearchIcon },
  { to: "/kits", label: "nav.kits", icon: BoxIcon },
  { to: "/installed", label: "nav.installed", icon: HardDriveDownloadIcon },
  { to: "/updates", label: "nav.updates", icon: RefreshCwIcon },
];

const SETTINGS: NavItem = { to: "/settings", label: "nav.settings", icon: SettingsIcon };

/**
 * The Sidebar, and its collapsed form the Rail.
 *
 * **One component in two states, not two components.** The Rail is 52px and icon-only;
 * the Sidebar is 180–400px and resizable. Below a 900px window width the Rail is
 * forced, and that is view state — it is never written to Workspace State, because a
 * narrow window must not permanently collapse the Sidebar.
 */
export function Sidebar({ onOpenPalette }: { onOpenPalette: () => void }) {
  const collapsed = useWorkspaceStore(isRail);
  const width = useWorkspaceStore((state) => state.workspace?.sidebar.width ?? 220);
  const setSidebarWidth = useWorkspaceStore((state) => state.setSidebarWidth);

  return (
    <div className="flex shrink-0">
      <nav
        aria-label={t("chrome.sidebarNav")}
        style={collapsed ? undefined : { width }}
        className={cn(
          "flex flex-col gap-1 overflow-hidden bg-bg p-2",
          "transition-[width] duration-[var(--motion-sidebar)] ease-standard",
          collapsed && "w-[var(--rail-width)]",
        )}
      >
        <SearchPill collapsed={collapsed} onClick={onOpenPalette} />

        <ul className="flex flex-col gap-0.5">
          {PRIMARY.map((item) => (
            <li key={item.to}>
              <NavLink item={item} collapsed={collapsed} />
            </li>
          ))}
        </ul>

        <ul className="mt-auto flex flex-col gap-0.5">
          <li>
            <NavLink item={SETTINGS} collapsed={collapsed} />
          </li>
        </ul>
      </nav>

      {collapsed ? (
        <div aria-hidden="true" className="w-px shrink-0 bg-border" />
      ) : (
        <SidebarResizer width={width} onWidth={setSidebarWidth} />
      )}
    </div>
  );
}

/** The Command Palette trigger: a magnifier, the word Search, and the shortcut hint. */
function SearchPill({ collapsed, onClick }: { collapsed: boolean; onClick: () => void }) {
  const label = t("palette.open");

  const button = (
    <button
      type="button"
      aria-label={label}
      onClick={onClick}
      className={cn(
        "mb-2 flex h-8 items-center gap-2 rounded-sm border border-border bg-surface",
        "text-sm text-text-muted transition-colors duration-100",
        "hover:border-text-muted hover:text-text",
        collapsed ? "justify-center px-0" : "px-2",
      )}
    >
      <SearchIcon className="size-4 shrink-0" aria-hidden="true" />
      {!collapsed && (
        <>
          <span className="flex-1 text-left">{label}</span>
          {/* Not aria-hidden: the button's aria-label already supplies the accessible
              name, so the hint is decoration to a screen reader either way — and
              aria-hidden on a descendant of a focusable element is its own problem. */}
          <kbd className="rounded-sm border border-border px-1 text-xs text-text-muted">
            {formatShortcut(SHORTCUTS.palette)}
          </kbd>
        </>
      )}
    </button>
  );

  // Rail items carry tooltips *and* aria-labels. Tooltips are not reliably announced
  // and are never the accessible name.
  return collapsed ? <Tooltip content={label}>{button}</Tooltip> : button;
}

function NavLink({ item, collapsed }: { item: NavItem; collapsed: boolean }) {
  const label = t(item.label);
  const Icon = item.icon;

  // `fuzzy` so /settings/appearance keeps Settings marked active.
  const active = useRouterState({
    select: (state) => state.location.pathname.startsWith(item.to),
  });

  const link = (
    <Link
      to={item.to}
      aria-label={collapsed ? label : undefined}
      aria-current={active ? "page" : undefined}
      className={cn(
        "relative flex h-8 items-center gap-2 rounded-sm text-sm",
        "transition-colors duration-100",
        collapsed ? "justify-center px-0" : "px-2",
        // Three cues, never one: a filled surface, a 500 weight, and a 2px Accent bar
        // on the leading edge. A tint this subtle is missable with full colour vision,
        // and under forced-colors the background is overridden entirely — which is why
        // the bar is a border rather than a background.
        active
          ? [
              "bg-surface font-medium text-text",
              "before:absolute before:inset-y-1 before:left-0 before:w-0.5",
              "before:rounded-full before:bg-accent",
              "forced-colors:before:bg-[Highlight]",
            ]
          : "text-text-muted hover:bg-surface hover:text-text",
      )}
    >
      <Icon className="size-4 shrink-0" aria-hidden="true" />
      {!collapsed && <span className="truncate">{label}</span>}
    </Link>
  );

  return collapsed ? <Tooltip content={label}>{link}</Tooltip> : link;
}
