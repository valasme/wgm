import { type ReactNode, useId } from "react";

import { cn } from "@/lib/cn";
import { useSettingsStore } from "@/stores/settings-store";

/**
 * One settings row.
 *
 * **Capped at 640px**, because at a 1200px window the Content Area is ~950px wide and a
 * full-bleed row puts a switch most of a metre from the label it belongs to. This is
 * the commonest layout mistake in desktop settings.
 *
 * The row owns the rollback's second channel: when an instant-apply write fails, the
 * message appears **here**, next to the control, and is wired to it with
 * `aria-describedby`. WCAG 3.3.1 wants the error next to the thing that failed, and a
 * toast that has already faded fails anyone who looked away.
 */
export function SettingRow({
  id,
  label,
  description,
  control,
}: {
  /** Also the key the store records a rollback error against. */
  id: string;
  label: string;
  description?: string;
  /** Given the ids to wire up, because only the control knows what to put them on. */
  control: (ids: { labelId: string; describedBy: string | undefined }) => ReactNode;
}) {
  const generated = useId();
  const labelId = `${generated}-label`;
  const descriptionId = `${generated}-description`;
  const errorId = `${generated}-error`;

  const error = useSettingsStore((state) => state.rowErrors[id]);

  const describedBy =
    [description ? descriptionId : null, error ? errorId : null].filter(Boolean).join(" ") ||
    undefined;

  return (
    <div className={cn("flex max-w-[var(--content-max-width)] flex-col gap-1 py-2")}>
      <div className="flex min-h-[var(--row-height)] items-center justify-between gap-6">
        <div className="flex flex-col gap-0.5">
          <span id={labelId} className="text-sm text-text">
            {label}
          </span>
          {description && (
            <span id={descriptionId} className="text-xs text-text-muted">
              {description}
            </span>
          )}
        </div>

        <div className="shrink-0">{control({ labelId, describedBy })}</div>
      </div>

      {error && (
        <p id={errorId} role="alert" className="text-xs text-text">
          {error}
        </p>
      )}
    </div>
  );
}
