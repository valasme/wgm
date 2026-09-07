import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { SIDEBAR_MAX_WIDTH, SIDEBAR_MIN_WIDTH, SidebarResizer } from "./sidebar-resizer";

/**
 * The splitter has two independent input paths and they fail independently — an earlier
 * version kept the keyboard working while the drag silently did nothing, because the
 * "am I dragging" flag was a ref and setting it never re-ran the effect that attaches
 * the move listeners. Both paths are asserted here for that reason.
 */
describe("the sidebar resizer", () => {
  function setup(width = 220) {
    const onWidth = vi.fn();
    render(<SidebarResizer width={width} onWidth={onWidth} />);
    return { onWidth, separator: screen.getByRole("separator") };
  }

  it("implements the ARIA splitter pattern", () => {
    const { separator } = setup(260);

    expect(separator).toHaveAttribute("aria-orientation", "vertical");
    expect(separator).toHaveAttribute("aria-valuenow", "260");
    expect(separator).toHaveAttribute("aria-valuemin", String(SIDEBAR_MIN_WIDTH));
    expect(separator).toHaveAttribute("aria-valuemax", String(SIDEBAR_MAX_WIDTH));
    // "260 pixels", not "260": the bare number tells a screen reader nothing.
    expect(separator).toHaveAttribute("aria-valuetext", "260 pixels");
    expect(separator).toHaveAttribute("tabindex", "0");
  });

  it("resizes on drag", () => {
    const { onWidth, separator } = setup();

    fireEvent.pointerDown(separator, { pointerId: 1 });
    fireEvent.pointerMove(window, { clientX: 300 });

    expect(onWidth).toHaveBeenCalledWith(300);
  });

  it("stops tracking the pointer once the button is released", () => {
    const { onWidth, separator } = setup();

    fireEvent.pointerDown(separator, { pointerId: 1 });
    fireEvent.pointerMove(window, { clientX: 300 });
    fireEvent.pointerUp(window);
    fireEvent.pointerMove(window, { clientX: 380 });

    expect(onWidth).toHaveBeenCalledTimes(1);
    expect(document.body.style.cursor).toBe("");
  });

  it("does not track the pointer before a drag starts", () => {
    const { onWidth } = setup();

    fireEvent.pointerMove(window, { clientX: 300 });

    expect(onWidth).not.toHaveBeenCalled();
  });

  it("clamps a drag to the bounds", () => {
    const { onWidth, separator } = setup();

    fireEvent.pointerDown(separator, { pointerId: 1 });
    fireEvent.pointerMove(window, { clientX: 4000 });
    fireEvent.pointerMove(window, { clientX: -50 });

    expect(onWidth).toHaveBeenNthCalledWith(1, SIDEBAR_MAX_WIDTH);
    expect(onWidth).toHaveBeenNthCalledWith(2, SIDEBAR_MIN_WIDTH);
  });

  it("releases the resize cursor when the OS takes the pointer away", () => {
    const { separator } = setup();

    fireEvent.pointerDown(separator, { pointerId: 1 });
    expect(document.body.style.cursor).toBe("col-resize");

    fireEvent.pointerCancel(window);

    expect(document.body.style.cursor).toBe("");
  });

  it("moves in 16px steps with the arrow keys", () => {
    const { onWidth, separator } = setup(220);

    fireEvent.keyDown(separator, { key: "ArrowRight" });
    expect(onWidth).toHaveBeenCalledWith(236);

    fireEvent.keyDown(separator, { key: "ArrowLeft" });
    expect(onWidth).toHaveBeenCalledWith(204);
  });

  it("jumps to the bounds with Home and End", () => {
    const { onWidth, separator } = setup(220);

    fireEvent.keyDown(separator, { key: "Home" });
    expect(onWidth).toHaveBeenCalledWith(SIDEBAR_MIN_WIDTH);

    fireEvent.keyDown(separator, { key: "End" });
    expect(onWidth).toHaveBeenCalledWith(SIDEBAR_MAX_WIDTH);
  });

  it("clamps a keyboard step at the bounds", () => {
    const { onWidth, separator } = setup(SIDEBAR_MAX_WIDTH);

    fireEvent.keyDown(separator, { key: "ArrowRight" });

    expect(onWidth).toHaveBeenCalledWith(SIDEBAR_MAX_WIDTH);
  });
});
