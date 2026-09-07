import { useRouterState } from "@tanstack/react-router";

/**
 * The route announcement.
 *
 * **Separate from, and additional to, the OS window title.** The title is what Alt+Tab
 * and the taskbar announce on *window* focus; this is what a screen reader hears when
 * the user navigates *within* the window, where the title change alone is not an event
 * anything reports.
 *
 * Focus also moves to the heading that changed — see `useHeadingFocus`. The two are
 * complementary: focus tells you where you are, the live region tells you that you
 * moved.
 */
export function RouteAnnouncer() {
  const pathname = useRouterState({ select: (state) => state.location.pathname });

  return (
    <div aria-live="polite" aria-atomic="true" className="sr-only">
      {pathname}
    </div>
  );
}
