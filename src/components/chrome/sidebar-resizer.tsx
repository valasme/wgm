import { useCallback, useEffect, useRef } from "react";

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
  const dragging = useRef(false);

  const clamp = useCallback(
    (value: number) => Math.min(SIDEBAR_MAX_WIDTH, Math.max(SIDEBAR_MIN_WIDTH, value)),
    [],
  );

  useEffect(() => {
    if (!dragging.current) {
      return;
    }

    const onMove = (event: PointerEvent) => onWidth(clamp(event.clientX));
    const onUp = () => {
      dragging.current = false;
      document.body.style.cursor = "";
    };

    window.addEventListener("pointermove", onMove);
    window.addEventListener("pointerup", onUp);

    return () => {
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("pointerup", onUp);
    };
  });

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
        dragging.current = true;
        document.body.style.cursor = "col-resize";
        event.currentTarget.setPointerCapture?.(event.pointerId);
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
      )}
    />
  );
}
