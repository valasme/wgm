import { t } from "@/i18n/t";
import { cn } from "@/lib/cn";

import { Button } from "../ui/button";

/**
 * The error state.
 *
 * **The correlation id is displayed, not just logged.** A user quoting `a3f9c1` is the
 * only way to find their failure in a Diagnostics Bundle; an id that exists solely in a
 * log file the user never opens is not a diagnostic. It is small, monospace and
 * selectable, and `data-selectable` opts it back out of the app-wide `user-select: none`.
 */
export function ErrorState({
  heading,
  body,
  correlationId,
  onRetry,
  retryLabel = t("state.retry"),
  className,
}: {
  heading: string;
  body?: string;
  correlationId?: string;
  onRetry?: () => void;
  retryLabel?: string;
  className?: string;
}) {
  return (
    <div
      role="alert"
      className={cn(
        "flex flex-1 flex-col items-center justify-center gap-3 p-12 text-center",
        className,
      )}
    >
      <h2 className="text-md font-medium text-text">{heading}</h2>

      {body && <p className="max-w-[52ch] text-sm text-text-muted">{body}</p>}

      {correlationId && (
        <code data-selectable className="text-xs text-text-muted">
          {t("crash.correlationId", { id: correlationId })}
        </code>
      )}

      {onRetry && (
        <Button onClick={onRetry} className="mt-2">
          {retryLabel}
        </Button>
      )}
    </div>
  );
}
