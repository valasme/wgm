import { beforeEach, describe, expect, it, vi } from "vitest";

// Imported as text rather than read with node:fs, so the app project does not have to
// pull Node's globals into its type environment for the sake of one test.
import bootScript from "../../public/boot.js?raw";

import {
  applyAppearance,
  BOOT_STORAGE_KEY,
  rememberAppearance,
  resolveColorScheme,
} from "./apply-theme";

function mockPrefersDark(matches: boolean) {
  vi.stubGlobal(
    "matchMedia",
    vi.fn((query: string) => ({
      matches: query.includes("dark") ? matches : false,
      media: query,
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
      dispatchEvent: vi.fn(),
    })),
  );
}

describe("resolveColorScheme", () => {
  it("passes an explicit scheme straight through", () => {
    mockPrefersDark(true);

    expect(resolveColorScheme("light")).toBe("light");
    expect(resolveColorScheme("dark")).toBe("dark");
    expect(resolveColorScheme("darker")).toBe("darker");
  });

  it("resolves system to light or dark, and never to darker", () => {
    mockPrefersDark(true);
    expect(resolveColorScheme("system")).toBe("dark");

    mockPrefersDark(false);
    expect(resolveColorScheme("system")).toBe("light");
  });
});

describe("applyAppearance", () => {
  it("writes all four attributes, and nothing else", () => {
    mockPrefersDark(false);
    const root = document.createElement("html");

    applyAppearance(
      { colorScheme: "darker", accent: "emerald", density: "compact", reduceMotion: "on" },
      root,
    );

    expect(root.dataset.theme).toBe("darker");
    expect(root.dataset.accent).toBe("emerald");
    expect(root.dataset.density).toBe("compact");
    expect(root.dataset.reduceMotion).toBe("on");
    expect(root.getAttribute("style")).toBeNull();
    expect(root.className).toBe("");
  });
});

describe("the appearance cache", () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it("round-trips through the key boot.js reads", () => {
    const appearance = {
      colorScheme: "dark",
      accent: "rose",
      density: "comfortable",
      reduceMotion: "system",
    } as const;

    rememberAppearance(appearance);

    expect(JSON.parse(localStorage.getItem(BOOT_STORAGE_KEY) ?? "null")).toEqual(appearance);
  });

  it("survives storage being unavailable", () => {
    const setItem = vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => {
      throw new Error("storage disabled");
    });

    expect(() =>
      rememberAppearance({
        colorScheme: "light",
        accent: "blue",
        density: "comfortable",
        reduceMotion: "system",
      }),
    ).not.toThrow();

    setItem.mockRestore();
  });
});

/**
 * `public/boot.js` is plain JavaScript that runs before the bundle, so nothing else in
 * the project imports it and nothing else would notice it drifting. These assertions
 * are the only thing holding it to the contract the TypeScript side depends on.
 */
describe("public/boot.js", () => {
  const source = bootScript;

  it("reads the same storage key the app writes", () => {
    expect(source).toContain(`"${BOOT_STORAGE_KEY}"`);
  });

  it("sets every attribute tokens.css keys off", () => {
    for (const attribute of ["theme", "accent", "density", "reduceMotion"]) {
      expect(source).toContain(`dataset.${attribute}`);
    }
  });

  it("never resolves system to darker", () => {
    expect(source).toContain('? "dark" : "light"');
  });

  it("registers the pre-React error handlers", () => {
    expect(source).toContain('addEventListener("error"');
    expect(source).toContain('addEventListener("unhandledrejection"');
  });

  it("keeps the three-second boot failsafe", () => {
    expect(source).toContain("3000");
    expect(source).toContain("show_window");
  });
});
