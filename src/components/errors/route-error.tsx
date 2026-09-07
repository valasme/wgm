import { useEffect } from "react";

import { t } from "@/i18n/t";
import { isAppError } from "@/ipc";
import { reportError } from "@/lib/report";

import { ErrorState } from "../states/error-state";

/**
 * The route-level boundary.
 *
 * It keeps the Title Bar and the Sidebar alive, so a failed page is somewhere the user
 * can navigate *away* from rather than a dead application — and it offers `reset`, the
 * boundary's own retry, not just a link elsewhere.
 *
 * **It logs explicitly.** A React error boundary catches its error, which means it
 * never reaches `window.onerror`; a boundary that only renders a fallback reports
 * nothing at all.
 */
export function RouteError({ error, reset }: { error: Error; reset?: () => void }) {
  useEffect(() => {
    reportError("route-boundary", error);
  }, [error]);

  return (
    <ErrorState
      heading={t("routeError.title")}
      body={t("routeError.body")}
      correlationId={isAppError(error) ? error.correlationId : undefined}
      onRetry={reset}
      retryLabel={t("routeError.retry")}
    />
  );
}
