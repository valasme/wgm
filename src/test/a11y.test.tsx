import { render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { axe } from "vitest-axe";
import { StepReady } from "@/components/onboarding/step-ready";
import { StepWelcome } from "@/components/onboarding/step-welcome";
import { SettingRow } from "@/components/settings/setting-row";
import { ShortcutSheet } from "@/components/shortcut-sheet";
import { EmptyState } from "@/components/states/empty-state";
import { ErrorState } from "@/components/states/error-state";
import { LoadingState } from "@/components/states/loading-state";
import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";
import { TooltipProvider } from "@/components/ui/tooltip";

/**
 * The axe pass.
 *
 * **This does not check contrast.** jsdom computes no layout, so axe's colour-contrast
 * rule silently no-ops and returns green — `scripts/check-contrast.ts` is the only
 * thing that verifies the palette, and reading a passing run here as contrast coverage
 * is the specific mistake docs/accessibility.md §11 exists to prevent.
 *
 * What it does check: roles, accessible names, labels, ARIA validity, landmarks and
 * heading order.
 */

function renderWithProviders(ui: React.ReactElement) {
  return render(<TooltipProvider>{ui}</TooltipProvider>);
}

beforeEach(() => {
  vi.stubGlobal(
    "matchMedia",
    vi.fn((query: string) => ({
      matches: false,
      media: query,
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
      dispatchEvent: vi.fn(),
    })),
  );
});

describe("state primitives", () => {
  it("an empty state has no violations", async () => {
    const { container } = renderWithProviders(
      <main>
        <EmptyState
          heading="No kits yet"
          body="A kit is a set of packages you install together."
          action={<Button variant="primary">Create a kit</Button>}
        />
      </main>,
    );

    await expect(axe(container)).resolves.toHaveNoViolations();
  });

  it("a loading state announces itself", async () => {
    const { container } = renderWithProviders(
      <main>
        <LoadingState />
      </main>,
    );

    expect(screen.getByRole("status")).toBeInTheDocument();
    await expect(axe(container)).resolves.toHaveNoViolations();
  });

  it("an error state has no violations and shows the correlation id", async () => {
    const { container } = renderWithProviders(
      <main>
        <h1>Page</h1>
        <ErrorState heading="This page didn't load" correlationId="a3f9c1" onRetry={() => {}} />
      </main>,
    );

    // Displayed, not just logged: a user quoting it is the only way to find their
    // failure in a Diagnostics Bundle.
    expect(screen.getByText(/a3f9c1/)).toBeInTheDocument();
    await expect(axe(container)).resolves.toHaveNoViolations();
  });
});

describe("settings rows", () => {
  it("a switch row is labelled and described", async () => {
    const { container } = renderWithProviders(
      <main>
        <h1>Settings</h1>
        <SettingRow
          id="general.restoreWindowPosition"
          label="Restore window position"
          description="Reopen at the size and place you left it."
          control={({ labelId, describedBy }) => (
            <Switch checked={false} aria-labelledby={labelId} aria-describedby={describedBy} />
          )}
        />
      </main>,
    );

    expect(screen.getByRole("switch", { name: "Restore window position" })).toBeInTheDocument();
    await expect(axe(container)).resolves.toHaveNoViolations();
  });
});

describe("onboarding steps", () => {
  it("the welcome step has no violations", async () => {
    const { container } = renderWithProviders(
      <main>
        <StepWelcome headingRef={null} />
      </main>,
    );

    await expect(axe(container)).resolves.toHaveNoViolations();
  });

  it("the ready step has no violations", async () => {
    const { container } = renderWithProviders(
      <main>
        <StepReady headingRef={null} />
      </main>,
    );

    await expect(axe(container)).resolves.toHaveNoViolations();
  });
});

describe("the shortcut sheet", () => {
  it("is reachable and has no violations", async () => {
    const { baseElement } = renderWithProviders(<ShortcutSheet open onOpenChange={() => {}} />);

    expect(screen.getByRole("dialog", { name: "Keyboard shortcuts" })).toBeInTheDocument();
    await expect(axe(baseElement)).resolves.toHaveNoViolations();
  });
});
