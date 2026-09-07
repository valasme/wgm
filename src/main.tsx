import { createHashHistory, createRouter, RouterProvider } from "@tanstack/react-router";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { toast } from "sonner";

import { RootBoundary } from "@/components/errors/root-boundary";
import { LoadingState } from "@/components/states/loading-state";
import { t } from "@/i18n/t";
import { commands } from "@/ipc";
import { installGlobalReporting, reportError } from "@/lib/report";
import { onRollback, useSettingsStore } from "@/stores/settings-store";
import { useWorkspaceStore } from "@/stores/workspace-store";
import { applyAppearance, watchSystemColorScheme } from "@/theme/apply-theme";

import { routeTree } from "./routeTree.gen";
import "./styles/global.css";

const router = createRouter({
  routeTree,
  // Hash history: a Tauri window is served from a custom protocol with no server
  // behind it, so a path-based history is a 404 waiting for the first deep link.
  history: createHashHistory(),
  defaultPendingComponent: LoadingState,
  scrollRestoration: true,
});

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}

installGlobalReporting();

/** The rollback announcement's third channel — the inline row error and the control
 * itself are the other two. See docs/accessibility.md §8. */
onRollback((_rowId, message) => {
  toast.error(t("settings.rollback"), { description: message });
});

async function boot() {
  // Both documents are loaded before the first render, so the shell never paints with
  // the wrong Appearance and then corrects itself.
  await Promise.all([useSettingsStore.getState().load(), useWorkspaceStore.getState().load()]);

  // Only matters while the Color Scheme preference is `system`, which is the default.
  watchSystemColorScheme(() => {
    const appearance = useSettingsStore.getState().settings?.appearance;
    if (appearance) {
      applyAppearance(appearance);
    }
  });

  const rootElement = document.getElementById("root");
  if (!rootElement) {
    throw new Error("#root is missing from index.html");
  }

  createRoot(rootElement, {
    // All three of React 19's root handlers, because a boundary catches its error and
    // so it never reaches window.onerror — and `onRecoverableError` catches
    // concurrent-rendering faults that no boundary ever sees.
    onUncaughtError: (error) => reportError("react:onUncaughtError", error),
    onCaughtError: (error) => reportError("react:onCaughtError", error),
    onRecoverableError: (error) => reportError("react:onRecoverableError", error),
  }).render(
    <StrictMode>
      <RootBoundary>
        <RouterProvider router={router} />
      </RootBoundary>
    </StrictMode>,
  );

  // First paint. The window is created hidden so boot never flashes an unstyled frame;
  // boot.js has a three-second failsafe in case this line is never reached.
  requestAnimationFrame(() => {
    clearTimeout(window.__wgmBootFailsafe);
    void commands.showWindow();
  });
}

void boot().catch((error) => {
  reportError("boot", error);
  // The static fallback in index.html is the last resort, and boot.js reveals it.
  window.dispatchEvent(new ErrorEvent("error", { message: String(error) }));
  void commands.showWindow();
});
