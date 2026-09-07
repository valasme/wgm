import { create } from "zustand";

import { t, tryTranslate } from "@/i18n/t";
import { type AppError, commands, type DocumentStatus, isAppError, type Settings } from "@/ipc";
import { applyAppearance, rememberAppearance } from "@/theme/apply-theme";

/**
 * Settings, mirrored from Rust.
 *
 * **Rust owns the value; this is a cache of it.** Every mutation goes through a command
 * and the store adopts whatever comes back, so what the UI shows is what is on disk.
 *
 * Instant apply, with rollback: the control moves immediately, Rust confirms, and on
 * failure the previous value is restored. The rollback is announced three ways per
 * docs/accessibility.md §8 — the control returns, an inline message appears on the row,
 * and a toast fires — because a silent revert lies to a screen reader that has already
 * read the new state.
 */

export interface SettingsState {
  settings: Settings | null;
  status: DocumentStatus | null;
  /** Keyed by settings row id. Rendered inline and wired up with `aria-describedby`. */
  rowErrors: Record<string, string>;
  loaded: boolean;

  load: () => Promise<void>;
  /**
   * Apply one change optimistically.
   *
   * `rowId` names the settings row, so a failure can be reported next to the control
   * that failed rather than only in a toast that has already faded.
   */
  apply: (
    rowId: string,
    optimistic: (settings: Settings) => Settings,
    commit: () => Promise<Settings>,
  ) => Promise<boolean>;
  clearRowError: (rowId: string) => void;
}

/** Fired on a rollback, so the toast layer can announce it without importing sonner here. */
export type RollbackListener = (rowId: string, message: string) => void;

const rollbackListeners = new Set<RollbackListener>();

export function onRollback(listener: RollbackListener): () => void {
  rollbackListeners.add(listener);
  return () => rollbackListeners.delete(listener);
}

function describe(error: unknown): string {
  return isAppError(error) ? messageFor(error) : t("errors.generic");
}

/**
 * Translate a Rust error.
 *
 * The code and its structured context are all that crossed the boundary; the sentence
 * is assembled here. An unrecognised code — one from a newer build — degrades to the
 * generic message rather than rendering a key at the user.
 */
export function messageFor(error: AppError): string {
  return tryTranslate(`errors.${error.code}`, error.context) ?? t("errors.generic");
}

export const useSettingsStore = create<SettingsState>((set, get) => ({
  settings: null,
  status: null,
  rowErrors: {},
  loaded: false,

  load: async () => {
    const [settings, status] = await Promise.all([
      commands.settingsGet(),
      commands.settingsStatus(),
    ]);

    applyAppearance(settings.appearance);
    rememberAppearance(settings.appearance);

    set({ settings, status, loaded: true });
  },

  apply: async (rowId, optimistic, commit) => {
    const previous = get().settings;

    if (previous === null) {
      return false;
    }

    // Optimistic: the control moves now. Appearance is applied here too, so a colour
    // scheme change is instant rather than waiting on a round trip.
    const candidate = optimistic(previous);
    applyAppearance(candidate.appearance);
    set((state) => ({
      settings: candidate,
      rowErrors: { ...state.rowErrors, [rowId]: "" },
    }));

    try {
      const confirmed = await commit();
      applyAppearance(confirmed.appearance);
      rememberAppearance(confirmed.appearance);
      set((state) => ({
        settings: confirmed,
        rowErrors: omit(state.rowErrors, rowId),
      }));
      return true;
    } catch (error) {
      // Roll back, and say so three ways.
      applyAppearance(previous.appearance);
      const message = describe(error);
      set((state) => ({
        settings: previous,
        rowErrors: { ...state.rowErrors, [rowId]: message },
      }));
      for (const listener of rollbackListeners) {
        listener(rowId, message);
      }
      // The status carries the coalesced write-failure count, which is what turns a
      // wall of toasts into one persistent banner.
      void commands.settingsStatus().then((status) => set({ status }));
      return false;
    }
  },

  clearRowError: (rowId) => set((state) => ({ rowErrors: omit(state.rowErrors, rowId) })),
}));

function omit(record: Record<string, string>, key: string): Record<string, string> {
  const { [key]: _removed, ...rest } = record;
  return rest;
}
