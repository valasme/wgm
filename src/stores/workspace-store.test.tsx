import { render } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { RAIL_BREAKPOINT, useIsRail } from "./workspace-store";

/**
 * The Rail state is read during render, not measured afterwards.
 *
 * Measuring in an effect means the shell commits one frame with the Sidebar expanded
 * and then corrects itself — which, with a width transition on the Sidebar, is the
 * visible flash-then-collapse on every launch in a narrow window.
 */

function resizeTo(width: number) {
  Object.defineProperty(window, "innerWidth", { value: width, configurable: true });
  window.dispatchEvent(new Event("resize"));
}

/** Every value the Sidebar was given, in commit order. */
function railFrames(): boolean[] {
  const frames: boolean[] = [];

  function Probe() {
    frames.push(useIsRail());
    return null;
  }

  render(<Probe />);
  return frames;
}

describe("the Rail", () => {
  it("is in effect on the first frame of a narrow window", () => {
    resizeTo(RAIL_BREAKPOINT - 100);

    const frames = railFrames();

    expect(frames[0]).toBe(true);
    // Not "ends up collapsed" — never expanded at all. One expanded frame is the flash.
    expect(frames).not.toContain(false);
  });
});
