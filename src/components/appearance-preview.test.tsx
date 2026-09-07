import { render } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { AppearancePreview } from "./appearance-preview";

/**
 * The preview's whole job is to show an Appearance the app is *not* currently in.
 * ADR-0002 makes the four `data-` attributes the entire theming contract, so what is
 * asserted here is that contract — jsdom computes no colour, and a preview that
 * inherited the app's own attributes would look identical for every option, which is
 * exactly the bug the accent swatches had.
 */

function renderPreview(ui: React.ReactElement, appScheme = "darker") {
  document.documentElement.dataset.theme = appScheme;
  const { container } = render(ui);
  return container.firstElementChild as HTMLElement;
}

describe("the Appearance preview", () => {
  it("renders in the scheme being previewed, not the one in effect", () => {
    const preview = renderPreview(
      <AppearancePreview colorScheme="light" accent="rose" density="comfortable" />,
    );

    expect(preview).toHaveAttribute("data-theme", "light");
  });

  it("resolves System to the scheme the OS is actually in", () => {
    // `system` is a preference, not a scheme: no token block matches it, so leaving it
    // on the element would paint the preview in the Light default whatever the OS says.
    vi.stubGlobal(
      "matchMedia",
      vi.fn((query: string) => ({
        matches: query.includes("dark"),
        media: query,
        addEventListener: vi.fn(),
        removeEventListener: vi.fn(),
        dispatchEvent: vi.fn(),
      })),
    );

    const preview = renderPreview(
      <AppearancePreview colorScheme="system" accent="blue" density="comfortable" />,
      "light",
    );

    expect(preview).toHaveAttribute("data-theme", "dark");
  });

  it("stays out of the accessibility tree", () => {
    // It is a picture of the app. Announcing its mock sidebar would put a second set of
    // navigation and buttons in front of a screen reader for every option on the page,
    // and the option's own label already says what is being chosen.
    const preview = renderPreview(
      <AppearancePreview colorScheme="dark" accent="blue" density="compact" />,
    );

    expect(preview).toHaveAttribute("aria-hidden", "true");
  });
});
