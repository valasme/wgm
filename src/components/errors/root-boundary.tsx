import { Component, type ErrorInfo, type ReactNode } from "react";

import { reportError } from "@/lib/report";

import { CrashScreen } from "./crash-screen";

/**
 * The outermost boundary. Below it is the crash screen and nothing else.
 *
 * **It logs explicitly**, because a boundary catches its error and so it never reaches
 * `window.onerror`. A boundary that only renders a fallback reports nothing — which is
 * the failure mode this class exists to avoid, not just the blank screen.
 *
 * A class component because React still has no hook for `componentDidCatch`.
 */
export class RootBoundary extends Component<{ children: ReactNode }, { error: unknown }> {
  state: { error: unknown } = { error: null };

  static getDerivedStateFromError(error: unknown) {
    return { error };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    reportError("root-boundary", error);
    reportError("root-boundary:componentStack", info.componentStack ?? "(none)");
  }

  render() {
    if (this.state.error !== null) {
      return <CrashScreen error={this.state.error} />;
    }

    return this.props.children;
  }
}
