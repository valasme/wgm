import type { MessageKey } from "@/i18n/t";
import { t } from "@/i18n/t";
import { formatShortcut, SHORTCUTS } from "@/lib/shortcuts";

import { Dialog, DialogContent, DialogHeader, DialogTitle } from "./ui/dialog";

/**
 * The keyboard shortcut sheet.
 *
 * `Ctrl+/` opens it, **and so does a link in Settings → About** — a keyboard-only route
 * to the list of keyboard shortcuts helps nobody who does not already know it.
 */
const ROWS: { keys: string; label: MessageKey }[] = [
  { keys: formatShortcut(SHORTCUTS.palette), label: "shortcuts.palette" },
  { keys: formatShortcut(SHORTCUTS.sidebar), label: "shortcuts.sidebar" },
  { keys: formatShortcut(SHORTCUTS.settings), label: "shortcuts.settings" },
  { keys: "Ctrl 1 – 4", label: "shortcuts.navigate" },
  { keys: formatShortcut(SHORTCUTS.help), label: "shortcuts.help" },
  { keys: "Esc", label: "shortcuts.close" },
  { keys: "Ctrl + / − / 0", label: "shortcuts.zoom" },
  { keys: formatShortcut(SHORTCUTS.systemMenu), label: "shortcuts.systemMenu" },
];

export function ShortcutSheet({
  open,
  onOpenChange,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t("shortcuts.title")}</DialogTitle>
        </DialogHeader>

        <dl className="flex flex-col gap-2">
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
      </DialogContent>
    </Dialog>
  );
}
