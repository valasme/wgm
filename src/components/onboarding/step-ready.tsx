import type { Ref } from "react";
import type { MessageKey } from "@/i18n/t";
import { t } from "@/i18n/t";
import { formatShortcut, SHORTCUTS } from "@/lib/shortcuts";

/**
 * Four shortcuts, not eight.
 *
 * The full list is `Ctrl+/` and Settings → About. What belongs here is the handful
 * worth learning on day one; a wall of bindings at the end of setup is a wall nobody
 * reads.
 */
const ROWS: { keys: string; label: MessageKey }[] = [
  { keys: formatShortcut(SHORTCUTS.palette), label: "shortcuts.palette" },
  { keys: formatShortcut(SHORTCUTS.sidebar), label: "shortcuts.sidebar" },
  { keys: formatShortcut(SHORTCUTS.settings), label: "shortcuts.settings" },
  { keys: formatShortcut(SHORTCUTS.help), label: "shortcuts.help" },
];

export function StepReady({ headingRef }: { headingRef: Ref<HTMLHeadingElement> }) {
  return (
    <div className="flex max-w-[52ch] flex-col items-center gap-6 text-center">
      <div className="flex flex-col gap-2">
        <h1 ref={headingRef} className="text-xl font-medium text-text outline-none">
          {t("onboarding.ready.title")}
        </h1>
        <p className="text-sm text-text-muted">{t("onboarding.ready.body")}</p>
      </div>

      <dl className="flex w-full flex-col gap-2 text-left">
        {ROWS.map((row) => (
          <div key={row.label} className="flex items-baseline justify-between gap-6">
            <dt className="text-sm text-text">{t(row.label)}</dt>
            <dd>
              <kbd className="rounded-sm border border-border px-1.5 py-0.5 text-xs text-text-muted">
                {row.keys}
              </kbd>
            </dd>
          </div>
        ))}
      </dl>
    </div>
  );
}
