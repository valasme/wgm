import { useCallback, useEffect, useState } from "react";

import { t } from "@/i18n/t";
import { cn } from "@/lib/cn";

export const SIDEBAR_MIN_WIDTH = 180;
export const SIDEBAR_MAX_WIDTH = 400;
/** Arrow keys move by this much. Home and End jump to the bounds. */
const KEYBOARD_STEP = 16;

/**
 * The Sidebar resizer.
 *
 * It is a splitter, and it implements the ARIA window-splitter pattern in full:
 * `role="separator"`, `aria-orientation`, a tab stop, `aria-valuenow`/`min`/`max`, and
 * an `aria-valuetext` that says "220 pixels" rather than "220".
 *
 * **The hit area is 10px, not 6px.** WCAG 2.5.8 asks for 24×24 and a splitter cannot be
 * that wide without changing the layout it exists to control, so the Essential
 * exception applies — but 6px is a hard target for anyone with a motor impairment, and
 * 10px costs nothing visually because the rendered line stays 1px.
 */
export function SidebarResizer({
  width,
  onWidth,
}: {
  width: number;
  onWidth: (width: number) => void;
}) {
  // **State, not a ref.** A ref set from `onPointerDown` causes no re-render, so the
  // effect below would never run again and the move listeners would never be attached
  // — the keyboard path would keep working while dragging silently did nothing.
  const [dragging, setDragging] = useState(false);

  const clamp = useCallback(
    (value: number) => Math.min(SIDEBAR_MAX_WIDTH, Math.max(SIDEBAR_MIN_WIDTH, value)),
    [],
  );

  useEffect(() => {
    if (!dragging) {
      return;
    }

    // On `window` rather than the separator: the pointer routinely leaves a 10px
    // target during a drag, and the drag has to survive that.
    const onMove = (event: PointerEvent) => onWidth(clamp(event.clientX));
    const stop = () => setDragging(false);

    document.body.style.cursor = "col-resize";
    window.addEventListener("pointermove", onMove);
    window.addEventListener("pointerup", stop);
    // A cancel fires when the OS takes the pointer away — a window switch, a
    // touch gesture. Without it the cursor stays `col-resize` for the session.
    window.addEventListener("pointercancel", stop);

    return () => {
      document.body.style.cursor = "";
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("pointerup", stop);
      window.removeEventListener("pointercancel", stop);
    };
  }, [dragging, clamp, onWidth]);

  return (
    // biome-ignore lint/a11y/useSemanticElements: an <hr> is a thematic break, not a control — it cannot carry a tabindex or the splitter's value attributes
    <div
      role="separator"
      aria-orientation="vertical"
      aria-label={t("chrome.sidebarWidth")}
      aria-valuenow={width}
      aria-valuemin={SIDEBAR_MIN_WIDTH}
      aria-valuemax={SIDEBAR_MAX_WIDTH}
      aria-valuetext={t("chrome.sidebarWidthValue", { width })}
      tabIndex={0}
      onPointerDown={(event) => {
        // Not `setPointerCapture`: capture would redirect pointermove to this element,
        // and the listeners are on `window` so the drag survives leaving the target.
        event.preventDefault();
        setDragging(true);
      }}
      onKeyDown={(event) => {
        const step = {
          ArrowLeft: -KEYBOARD_STEP,
          ArrowRight: KEYBOARD_STEP,
        }[event.key];

        if (step !== undefined) {
          event.preventDefault();
          onWidth(clamp(width + step));
          return;
        }

        if (event.key === "Home") {
          event.preventDefault();
          onWidth(SIDEBAR_MIN_WIDTH);
        } else if (event.key === "End") {
          event.preventDefault();
          onWidth(SIDEBAR_MAX_WIDTH);
        }
      }}
      className={cn(
        // 10px of hit area, 1px of line. The rendered divider is the ::after.
        "group relative w-[var(--resizer-hit-area)] shrink-0 cursor-col-resize",
        "-ml-[calc(var(--resizer-hit-area)/2)]",
        "after:absolute after:inset-y-0 after:left-1/2 after:w-px",
        "after:bg-border after:transition-colors after:duration-100",
        "hover:after:bg-text-muted",
        "focus-visible:after:bg-accent",
        dragging && "after:bg-accent",
      )}
    />
  );
}
