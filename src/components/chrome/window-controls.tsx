import * as Toolbar from "@radix-ui/react-toolbar";
import { useEffect, useRef, useState } from "react";

import { t } from "@/i18n/t";
import { commands } from "@/ipc";
import { cn } from "@/lib/cn";
import { getWindow } from "@/lib/window";

/**
 * Minimise, maximise/restore and close.
 *
 * **Hand-drawn 10×10 SVGs on `currentColor`, not lucide.** These have to match the
 * metrics of Segoe Fluent Icons, because they sit where Windows' own controls sit and
 * a user's eye knows exactly what belongs there. lucide's rounded `Square` reads
 * visibly wrong as a maximise button. `currentColor` rather than a fill, so the glyphs
 * survive `forced-colors: active` — a hardcoded fill disappears there.
 *
 * **One `role="toolbar"`, one tab stop, arrow keys between the three.** And **DOM order
 * matching visual order**: an earlier draft put these last in the DOM and moved them
 * with CSS, which is a WCAG 2.4.3 failure for sighted keyboard users, who would watch
 * focus jump from the bottom of the page to the top-right corner.
 *
 * The button geometry is reported to Rust, because `WM_NCHITTEST` has to answer
 * `HTMAXBUTTON` over the maximise button for Snap Layouts to appear on hover, and Rust
 * cannot know where a React component ended up.
 */
export function WindowControls() {
  const [maximized, setMaximized] = useState(false);
  const maximizeRef = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    const window = getWindow();
    if (!window) {
      return;
    }

    let cancelled = false;

    const sync = async () => {
      const isMaximized = await window.isMaximized();
      if (!cancelled) {
        setMaximized(isMaximized);
      }
    };

    void sync();
    const unlisten = window.onResized(() => void sync());

    return () => {
      cancelled = true;
      void unlisten.then((stop) => stop());
    };
  }, []);

  // Report where the maximise button ended up, in physical pixels. Re-reported on any
  // resize, because the Title Bar is right-aligned and the button moves with the window.
  useEffect(() => {
    const report = () => {
      const element = maximizeRef.current;
      if (!element) {
        return;
      }

      const bounds = element.getBoundingClientRect();
      const scale = window.devicePixelRatio || 1;

      void commands.setMaximizeButtonRect({
        x: Math.round(bounds.left * scale),
        y: Math.round(bounds.top * scale),
        width: Math.round(bounds.width * scale),
        height: Math.round(bounds.height * scale),
      });
    };

    report();
    window.addEventListener("resize", report);
    return () => window.removeEventListener("resize", report);
  }, []);

  return (
    <Toolbar.Root
      aria-label={t("chrome.windowControls")}
      // Not draggable: this sits inside a `data-tauri-drag-region` bar, and without
      // this the buttons would start a window drag instead of activating.
      data-tauri-drag-region={false}
      className="flex h-full items-stretch"
    >
      <ControlButton label={t("chrome.minimize")} onClick={() => void getWindow()?.minimize()}>
        <MinimizeGlyph />
      </ControlButton>

      <ControlButton
        ref={maximizeRef}
        label={maximized ? t("chrome.restore") : t("chrome.maximize")}
        onClick={() => void getWindow()?.toggleMaximize()}
      >
        {maximized ? <RestoreGlyph /> : <MaximizeGlyph />}
      </ControlButton>

      <ControlButton
        label={t("chrome.close")}
        onClick={() => void getWindow()?.close()}
        className="hover:bg-[#c42b1c] hover:text-white forced-colors:hover:bg-[Highlight]"
      >
        <CloseGlyph />
      </ControlButton>
    </Toolbar.Root>
  );
}

function ControlButton({
  label,
  children,
  className,
  onClick,
  ref,
}: {
  label: string;
  children: React.ReactNode;
  className?: string;
  onClick: () => void;
  ref?: React.Ref<HTMLButtonElement>;
}) {
  return (
    <Toolbar.Button
      ref={ref}
      type="button"
      aria-label={label}
      onClick={onClick}
      className={cn(
        // 46×36 is what Windows uses. The 10×10 artwork sits inside a target well
        // above the 24×24 floor, in Compact density too.
        "flex h-full w-[46px] items-center justify-center",
        "text-text transition-colors duration-100",
        "hover:bg-surface",
        className,
      )}
    >
      {children}
    </Toolbar.Button>
  );
}

/** All four glyphs are 10×10 on a 1px stroke, matching Segoe Fluent Icons' metrics. */
const glyph = {
  width: 10,
  height: 10,
  viewBox: "0 0 10 10",
  fill: "none",
  stroke: "currentColor",
  strokeWidth: 1,
} as const;

function MinimizeGlyph() {
  return (
    <svg {...glyph} aria-hidden="true">
      <path d="M0 5.5h10" />
    </svg>
  );
}

function MaximizeGlyph() {
  return (
    <svg {...glyph} aria-hidden="true">
      <rect x="0.5" y="0.5" width="9" height="9" />
    </svg>
  );
}

function RestoreGlyph() {
  return (
    <svg {...glyph} aria-hidden="true">
      <rect x="0.5" y="2.5" width="7" height="7" />
      <path d="M2.5 2.5v-2h7v7h-2" />
    </svg>
  );
}

function CloseGlyph() {
  return (
    <svg {...glyph} aria-hidden="true">
      <path d="M0.5 0.5l9 9M9.5 0.5l-9 9" />
    </svg>
  );
}
