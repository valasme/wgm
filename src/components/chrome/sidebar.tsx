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
import { useIsRail, useWorkspaceStore } from "@/stores/workspace-store";
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
 * Every item in the Rail is a square the width of the Rail's content box, so the icons
 * sit on one vertical axis — the same axis the Title Bar's sidebar toggle sits on.
 * Expanded, the row height follows density like every other row in the app.
 */
const ITEM = "flex shrink-0 items-center rounded-sm text-sm transition-colors duration-100";
const ITEM_RAIL = "size-[var(--rail-item)] justify-center";
const ITEM_WIDE = "h-[var(--row-height)] w-full gap-2 px-2";

/**
 * The Sidebar, and its collapsed form the Rail.
 *
 * **One component in two states, not two components.** The Rail is 52px and icon-only;
 * the Sidebar is 180–400px and resizable. Below a 900px window width the Rail is
 * forced, and that is view state — it is never written to Workspace State, because a
 * narrow window must not permanently collapse the Sidebar.
 */
export function Sidebar({ onOpenPalette }: { onOpenPalette: () => void }) {
  const collapsed = useIsRail();
  const width = useWorkspaceStore((state) => state.workspace?.sidebar.width ?? 220);
  const setSidebarWidth = useWorkspaceStore((state) => state.setSidebarWidth);

  return (
    <div className="flex shrink-0">
      <nav
        aria-label={t("chrome.sidebarNav")}
        style={collapsed ? undefined : { width }}
        className={cn(
          "flex flex-col gap-1 overflow-hidden bg-bg p-[var(--rail-padding)]",
          "transition-[width] duration-[var(--motion-sidebar)] ease-standard",
          collapsed && "w-[var(--rail-width)]",
        )}
      >
        <SearchItem collapsed={collapsed} onClick={onOpenPalette} />

        <ul className="flex flex-col gap-0.5">
          {PRIMARY.map((item) => (
            <li key={item.to} className="flex">
              <NavLink item={item} collapsed={collapsed} />
            </li>
          ))}
        </ul>

        <ul className="mt-auto flex flex-col gap-0.5">
          <li className="flex">
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

/**
 * The Command Palette trigger.
 *
 * Expanded it is a search field's twin — a border, the word Search and the shortcut.
 * In the Rail it is one more icon on the same axis as the rest: a bordered box among
 * borderless ones was the thing that made the Rail look assembled from two kits.
 */
function SearchItem({ collapsed, onClick }: { collapsed: boolean; onClick: () => void }) {
  const label = t("palette.open");

  const button = (
    <button
      type="button"
      aria-label={label}
      onClick={onClick}
      className={cn(
        ITEM,
        "mb-1 text-text-muted",
        collapsed
          ? [ITEM_RAIL, "hover:bg-surface hover:text-text"]
          : [
              ITEM_WIDE,
              "border border-border bg-surface hover:border-text-muted hover:text-text",
            ],
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
        ITEM,
        collapsed ? ITEM_RAIL : ITEM_WIDE,
        // Two cues, not one: a filled surface and a 500 weight. There is no Accent bar
        // on the leading edge — it was decoration on a control that already reads as
        // selected. Under forced-colors the surface *is* overridden, so a border comes
        // back there and only there, because that mode has no other cue left.
        active
          ? [
              "bg-surface font-medium text-text",
              "forced-colors:border forced-colors:border-[Highlight]",
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
