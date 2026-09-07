import { Component, type ErrorInfo, type ReactNode } from "react";

import { t } from "@/i18n/t";
import { isAppError } from "@/ipc";
import { reportError } from "@/lib/report";

import { ErrorState } from "../states/error-state";

/**
 * The section boundary — the innermost of the four layers.
 *
 * One settings panel can fail without taking the page with it, and the retry is scoped
 * to that panel. Recent problems reads a ring buffer and the bundle preview builds a
 * zip; either can fail on its own, and neither is a reason to lose the whole of
 * Settings.
 *
 * Like every other boundary it **logs explicitly**: a caught error never reaches
 * `window.onerror`.
 */
export class SectionBoundary extends Component<
  { children: ReactNode; name: string },
  { error: unknown; attempt: number }
> {
  state: { error: unknown; attempt: number } = { error: null, attempt: 0 };

  static getDerivedStateFromError(error: unknown) {
    return { error };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    reportError(`section-boundary:${this.props.name}`, error);
    reportError(
      `section-boundary:${this.props.name}:componentStack`,
      info.componentStack ?? "(none)",
    );
  }

  render() {
    if (this.state.error !== null) {
      return (
        <ErrorState
          heading={t("sectionError.title")}
          correlationId={
            isAppError(this.state.error) ? this.state.error.correlationId : undefined
          }
          retryLabel={t("sectionError.retry")}
          // Remounting the subtree by key is the retry: resetting the flag alone would
          // re-render the same broken children and fail again immediately.
          onRetry={() =>
            this.setState((state) => ({ error: null, attempt: state.attempt + 1 }))
          }
          className="py-8"
        />
      );
    }

    return <div key={this.state.attempt}>{this.props.children}</div>;
  }
}
