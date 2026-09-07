import { useEffect, useRef } from "react";
import type { MessageKey } from "@/i18n/t";
import { t } from "@/i18n/t";
import { getWindow } from "@/lib/window";

/**
 * Focus and the window title on navigation.
 *
 * **Focus moves to the heading that changed** — the `<h2>` inside Settings, the `<h1>`
 * everywhere else. Sending focus to the `<h1>` on a Settings sub-route change would
 * announce "Settings" four times in a row and never name the page the user opened.
 *
 * **The OS window title is a real accessibility surface.** Alt+Tab, the taskbar and
 * every screen reader announce it on window focus, and a single-page app does not
 * update it on navigation unless told to. `<title>` is kept in step for the same
 * reason, and both are separate from — and additional to — the live-region
 * announcement.
 */

/** Move focus to a heading element, without leaving it in the tab order. */
export function focusHeading(element: HTMLElement | null): void {
  if (!element) {
    return;
  }

  // -1 rather than 0: the heading is a focus *target*, not a tab stop.
  element.setAttribute("tabindex", "-1");
  element.focus({ preventScroll: false });
}

/**
 * Give a route heading a ref that takes focus on mount.
 *
 * One hook per heading rather than a router-level effect, because "the heading that
 * changed" is something only the heading knows.
 */
export function useHeadingFocus<T extends HTMLElement>() {
  const ref = useRef<T>(null);

  useEffect(() => {
    focusHeading(ref.current);
  }, []);

  return ref;
}

/** Set the OS window title and `<title>` together. */
export function useWindowTitle(page: MessageKey): void {
  useEffect(() => {
    const title = t("app.windowTitle", { page: t(page) });

    document.title = title;
    void getWindow()?.setTitle(title);
  }, [page]);
}
