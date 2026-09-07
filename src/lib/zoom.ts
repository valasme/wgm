import { useEffect } from "react";

import { getWebview } from "@/lib/window";

/**
 * `Ctrl` `+` / `-` / `0` zoom.
 *
 * **Off by default in Tauri, and enabled here explicitly.** It is the cheapest
 * low-vision accommodation available, and there is no browser chrome to offer it
 * otherwise. Nothing in wgm has a fixed height on a text container, so a 1.5× override
 * does not clip.
 */
const STEPS = [0.67, 0.75, 0.8, 0.9, 1, 1.1, 1.25, 1.5, 1.75, 2];
const DEFAULT_INDEX = STEPS.indexOf(1);

export function useZoom(): void {
  useEffect(() => {
    let index = DEFAULT_INDEX;

    const applyZoom = () => {
      const factor = STEPS[index] ?? 1;
      void getWebview()?.setZoom(factor);
    };

    const onKeyDown = (event: KeyboardEvent) => {
      if (!event.ctrlKey || event.altKey) {
        return;
      }

      // `=` because Ctrl+Plus is Ctrl+Shift+Equals on most layouts, and users press
      // both; `NumpadAdd` covers the keypad.
      if (event.key === "+" || event.key === "=" || event.code === "NumpadAdd") {
        event.preventDefault();
        index = Math.min(STEPS.length - 1, index + 1);
        applyZoom();
      } else if (event.key === "-" || event.code === "NumpadSubtract") {
        event.preventDefault();
        index = Math.max(0, index - 1);
        applyZoom();
      } else if (event.key === "0" || event.code === "Numpad0") {
        event.preventDefault();
        index = DEFAULT_INDEX;
        applyZoom();
      }
    };

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, []);
}
