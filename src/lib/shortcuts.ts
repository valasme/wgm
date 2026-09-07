import { useEffect } from "react";

/**
 * Keyboard shortcuts.
 *
 * Two absences matter as much as the presences:
 *
 * - **`F5` and `Ctrl+R` are disabled in release.** A reload blanks the window and is
 *   indistinguishable from a crash, and there is no address bar to explain otherwise.
 * - **Devtools are disabled in release too**, for the same reason — shipping them while
 *   disabling reload would be contradictory.
 *
 * `Ctrl` `+`/`-`/`0` zoom is enabled explicitly. It is off by default in Tauri, and it
 * is the cheapest low-vision accommodation available.
 */

export interface Shortcut {
  key: string;
  ctrl?: boolean;
  shift?: boolean;
}

export const SHORTCUTS = {
  palette: { key: "k", ctrl: true },
  sidebar: { key: "b", ctrl: true },
  settings: { key: ",", ctrl: true },
  help: { key: "/", ctrl: true },
  systemMenu: { key: " ", alt: true },
} as const satisfies Record<string, Shortcut & { alt?: boolean }>;

/** `Ctrl+1` … `Ctrl+4` go to the first four navigation entries. */
export const NAV_SHORTCUT_TARGETS = ["/search", "/kits", "/installed", "/updates"] as const;

/** Render a shortcut the way Windows writes it: `Ctrl K`. */
export function formatShortcut(shortcut: Shortcut & { alt?: boolean }): string {
  const parts: string[] = [];
  if (shortcut.ctrl) parts.push("Ctrl");
  if (shortcut.alt) parts.push("Alt");
  if (shortcut.shift) parts.push("Shift");
  parts.push(shortcut.key === " " ? "Space" : shortcut.key.toUpperCase());
  return parts.join(" ");
}

function matches(event: KeyboardEvent, shortcut: Shortcut & { alt?: boolean }): boolean {
  return (
    event.key.toLowerCase() === shortcut.key.toLowerCase() &&
    event.ctrlKey === Boolean(shortcut.ctrl) &&
    event.altKey === Boolean(shortcut.alt) &&
    event.shiftKey === Boolean(shortcut.shift)
  );
}

/** Is the user typing? A shortcut must not steal a keystroke from a text field. */
function isTyping(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) {
    return false;
  }

  return (
    target.tagName === "INPUT" ||
    target.tagName === "TEXTAREA" ||
    target.isContentEditable ||
    target.getAttribute("role") === "textbox"
  );
}

export interface ShortcutHandlers {
  onPalette?: () => void;
  onToggleSidebar?: () => void;
  onSettings?: () => void;
  onHelp?: () => void;
  onSystemMenu?: () => void;
  onNavigate?: (path: (typeof NAV_SHORTCUT_TARGETS)[number]) => void;
}

/** Register the global shortcuts for as long as the component is mounted. */
export function useShortcuts(handlers: ShortcutHandlers): void {
  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      // Reload is disabled everywhere, not just in release: a developer who reloads by
      // habit gets the same blank-window confusion a user would.
      if (event.key === "F5" || (event.ctrlKey && event.key.toLowerCase() === "r")) {
        event.preventDefault();
        return;
      }

      if (matches(event, SHORTCUTS.systemMenu)) {
        event.preventDefault();
        handlers.onSystemMenu?.();
        return;
      }

      // Everything below is a Ctrl chord, and none of them are things a text field
      // wants — but Ctrl+K in particular is a readline binding people have muscle
      // memory for, so typing wins.
      if (isTyping(event.target) && event.key !== "Escape") {
        return;
      }

      if (matches(event, SHORTCUTS.palette)) {
        event.preventDefault();
        handlers.onPalette?.();
        return;
      }

      if (matches(event, SHORTCUTS.sidebar)) {
        event.preventDefault();
        handlers.onToggleSidebar?.();
        return;
      }

      if (matches(event, SHORTCUTS.settings)) {
        event.preventDefault();
        handlers.onSettings?.();
        return;
      }

      if (matches(event, SHORTCUTS.help)) {
        event.preventDefault();
        handlers.onHelp?.();
        return;
      }

      if (event.ctrlKey && !event.altKey && !event.shiftKey) {
        const index = Number.parseInt(event.key, 10) - 1;
        const target = NAV_SHORTCUT_TARGETS[index];
        if (target !== undefined) {
          event.preventDefault();
          handlers.onNavigate?.(target);
        }
      }
    };

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [handlers]);
}
