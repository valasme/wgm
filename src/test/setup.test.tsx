import { render } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { axe } from "vitest-axe";

// This file tests the test harness, not the product. It exists because vitest-axe
// ships its matcher augmentation for a Vitest version we are not on: if the wiring in
// setup.ts silently stops applying, every a11y test in the suite passes vacuously.
describe("test harness", () => {
  it("registers the axe matcher", () => {
    expect(expect(undefined)).toHaveProperty("toHaveNoViolations");
  });

  it("reports a violation rather than passing silently", async () => {
    const { container } = render(
      // biome-ignore lint/a11y/useAltText: the missing alt text is the point of the test
      <img src="data:image/gif;base64,R0lGODlhAQABAAAAACw=" />,
    );

    const results = await axe(container);

    expect(Object.values(results.violations).length).toBeGreaterThan(0);
  });

  it("passes an accessible tree", async () => {
    const { container } = render(
      <main>
        <h1>wgm</h1>
      </main>,
    );

    await expect(axe(container)).resolves.toHaveNoViolations();
  });
});
