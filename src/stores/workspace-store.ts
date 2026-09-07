import { useSyncExternalStore } from "react";
import { create } from "zustand";

import { commands, type DocumentStatus, type WorkspaceState } from "@/ipc";

/**
 * Workspace State, mirrored from Rust.
 *
 * Never exported and never imported — importing someone else's settings must not move
 * your window or skip your onboarding, and that exclusion is structural rather than a
 * list someone maintains.
 *
 * The Sidebar width is the one field written at pointer speed, so it updates locally on
 * every frame of a drag and is persisted on a 300 ms debounce. Sending sixty writes a
 * second through a single global write lock would be a different kind of bug.
 */

/** Below this window width the Sidebar is forced to the Rail. */
export const RAIL_BREAKPOINT = 900;

const SIDEBAR_WRITE_DEBOUNCE_MS = 300;

export interface WorkspaceStoreState {
  workspace: WorkspaceState | null;
  status: DocumentStatus | null;
  loaded: boolean;

  load: () => Promise<void>;
  setSidebarWidth: (width: number) => void;
  setSidebarCollapsed: (collapsed: boolean) => Promise<void>;
  completeOnboarding: () => Promise<void>;
  replayOnboarding: () => Promise<void>;
}

let widthTimer: ReturnType<typeof setTimeout> | undefined;

export const useWorkspaceStore = create<WorkspaceStoreState>((set, get) => ({
  workspace: null,
  status: null,
  loaded: false,

  load: async () => {
    const [workspace, status] = await Promise.all([
      commands.workspaceGet(),
      commands.workspaceStatus(),
    ]);
    set({ workspace, status, loaded: true });
  },

  setSidebarWidth: (width) => {
    const current = get().workspace;
    if (current === null) {
      return;
    }

    // Live, every frame.
    set({ workspace: { ...current, sidebar: { ...current.sidebar, width } } });

    // Persisted, once the drag settles.
    clearTimeout(widthTimer);
    widthTimer = setTimeout(() => {
      void commands.workspaceSetSidebarWidth(width);
    }, SIDEBAR_WRITE_DEBOUNCE_MS);
  },

  setSidebarCollapsed: async (collapsed) => {
    const current = get().workspace;
    if (current === null) {
      return;
    }

    set({ workspace: { ...current, sidebar: { ...current.sidebar, collapsed } } });

    // A deliberate collapse is recorded. A Rail forced by a narrow window is not — see
    // `railForced`, which is view state and never reaches the document.
    const result = await commands.workspaceSetSidebarCollapsed(collapsed);
    if (result.status === "ok") {
      set({ workspace: result.data });
    }
  },

  completeOnboarding: async () => {
    const result = await commands.workspaceCompleteOnboarding();
    if (result.status === "ok") {
      set({ workspace: result.data });
    }
    // A failure here is deliberately not surfaced as an error: onboarding finished, and
    // the Ephemeral Mode banner already explains why nothing is being saved. Replaying
    // setup on every launch would be a maddening symptom for a small failure.
  },

  replayOnboarding: async () => {
    const result = await commands.workspaceReplayOnboarding();
    if (result.status === "ok") {
      set({ workspace: result.data });
    }
  },
}));

/**
 * Is the window too narrow for the Sidebar?
 *
 * Read during render rather than measured in an effect. An effect runs *after* the
 * first frame is committed, so the shell would paint the Sidebar expanded and then
 * animate it closed — the flash on every launch in a narrow window.
 */
export function useRailForced(): boolean {
  return useSyncExternalStore(subscribeToResize, () => window.innerWidth < RAIL_BREAKPOINT);
}

/** Is the Sidebar showing as a Rail right now, for whatever reason? */
export function useIsRail(): boolean {
  const collapsed = useWorkspaceStore((state) => state.workspace?.sidebar.collapsed ?? false);

  return useRailForced() || collapsed;
}

function subscribeToResize(onChange: () => void): () => void {
  window.addEventListener("resize", onChange);
  return () => window.removeEventListener("resize", onChange);
}
