import { createFileRoute, Link, Outlet, useRouterState } from "@tanstack/react-router";

import { RouteError } from "@/components/errors/route-error";
import type { MessageKey } from "@/i18n/t";
import { t } from "@/i18n/t";
import { cn } from "@/lib/cn";
import { useWindowTitle } from "@/lib/focus";

export const Route = createFileRoute("/_app/settings")({
  component: SettingsLayout,
  errorComponent: RouteError,
});

const PAGES: { to: string; label: MessageKey }[] = [
  { to: "/settings/appearance", label: "settings.appearance.title" },
  { to: "/settings/general", label: "settings.general.title" },
  { to: "/settings/advanced", label: "settings.advanced.title" },
  { to: "/settings/about", label: "settings.about.title" },
];

/**
 * The Settings layout.
 *
 * It owns the `<h1>`; each of the four pages owns an `<h2>`, and focus on navigation
 * moves to the heading that changed. Rows sit in from the divider and cap at 640px —
 * see `SettingRow` for why.
 *
 * Four pages, not five: the copy and the grouping are fixed in docs/design.md §8.
 */
function SettingsLayout() {
  useWindowTitle("settings.title");

  const pathname = useRouterState({ select: (state) => state.location.pathname });

  return (
    <div className="flex flex-1 flex-col p-[var(--page-padding)]">
      <h1 className="mb-4 text-lg font-medium text-text">{t("settings.title")}</h1>

      <nav
        aria-label={t("settings.title")}
        className="mb-[var(--section-gap)] flex gap-1 border-b border-border"
      >
        {PAGES.map((page) => {
          const active = pathname.startsWith(page.to);

          return (
            <Link
              key={page.to}
              to={page.to}
              aria-current={active ? "page" : undefined}
              className={cn(
                "-mb-px border-b-2 px-3 py-2 text-sm transition-colors duration-100",
                active
                  ? "border-accent font-medium text-text"
                  : "border-transparent text-text-muted hover:text-text",
              )}
            >
              {t(page.label)}
            </Link>
          );
        })}
      </nav>

      <Outlet />
    </div>
  );
}
