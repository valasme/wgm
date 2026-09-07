import { describe, expect, it } from "vitest";

import { en } from "./en";
import { t, tryTranslate } from "./t";

describe("t", () => {
  it("returns a message with no placeholders unchanged", () => {
    expect(t("route.kits.heading")).toBe("No kits yet");
  });

  it("interpolates typed parameters", () => {
    expect(t("app.windowTitle", { page: "Settings" })).toBe("Settings · wgm");
    expect(t("chrome.sidebarWidthValue", { width: 220 })).toBe("220 pixels");
  });

  it("interpolates every occurrence of every placeholder", () => {
    expect(
      t("errors.DOCUMENT_CORRUPT", { document: "settings", backup: "settings.corrupt.json" }),
    ).toBe("settings couldn't be read. Your previous file is saved as settings.corrupt.json.");
  });
});

describe("tryTranslate", () => {
  it("looks up a key only known at runtime", () => {
    expect(tryTranslate("errors.IPC_UNKNOWN")).toBe("Something went wrong inside wgm.");
  });

  it("returns undefined for an unknown key rather than throwing", () => {
    // A code from a newer build must degrade to the generic message, not take the
    // screen down on its way past.
    expect(tryTranslate("errors.CODE_FROM_A_FUTURE_RELEASE")).toBeUndefined();
  });

  it("interpolates when it can and leaves the placeholder visible when it cannot", () => {
    expect(tryTranslate("errors.EXPORT_FAILED", { path: "D:\\out.json" })).toBe(
      "wgm couldn't write to D:\\out.json.",
    );
    // Visible beats "undefined": it says plainly that something is missing rather than
    // asserting a value that is not there.
    expect(tryTranslate("errors.EXPORT_FAILED")).toBe("wgm couldn't write to {path}.");
  });
});

describe("the catalog", () => {
  it("has an entry for every error code Rust can send", () => {
    // Mirrors ErrorCode::ALL in src-tauri/src/error.rs. A code added there without a
    // message here would reach a user as the generic sentence.
    const codes = [
      "STORAGE_UNAVAILABLE",
      "DOCUMENT_READ_FAILED",
      "DOCUMENT_WRITE_FAILED",
      "DOCUMENT_CORRUPT",
      "DOCUMENT_SCHEMA_TOO_NEW",
      "DOCUMENT_MIGRATION_FAILED",
      "STORAGE_EPHEMERAL",
      "IMPORT_INVALID",
      "IMPORT_SCHEMA_TOO_NEW",
      "IMPORT_BACKUP_FAILED",
      "EXPORT_FAILED",
      "DIAGNOSTICS_BUNDLE_FAILED",
      "DIAGNOSTICS_REDACTION_UNSAFE",
      "RELEASE_CHECK_OFFLINE",
      "RELEASE_CHECK_SERVER_ERROR",
      "RELEASE_CHECK_RATE_LIMITED",
      "RELEASE_CHECK_TIMEOUT",
      "RELEASE_CHECK_MALFORMED",
      "WINDOW_OPERATION_FAILED",
      "AUTOSTART_FAILED",
      "IPC_UNKNOWN",
      "DEBUG_FORCED",
    ];

    for (const code of codes) {
      expect(tryTranslate(`errors.${code}`), `errors.${code} is missing`).toBeDefined();
    }
  });

  it("has an entry for every path note", () => {
    for (const note of [
      "PORTABLE_FOLDER_READ_ONLY",
      "APP_DATA_UNAVAILABLE",
      "APP_DATA_READ_ONLY",
    ]) {
      expect(tryTranslate(`pathNote.${note}`), `pathNote.${note} is missing`).toBeDefined();
    }
  });

  it("contains no empty messages", () => {
    for (const [key, message] of Object.entries(en)) {
      expect(message.trim(), `${key} is empty`).not.toBe("");
    }
  });

  it("does not apologise", () => {
    // Errors state what happened and what to do. docs/design.md §8.
    for (const [key, message] of Object.entries(en)) {
      expect(message.toLowerCase(), `${key} apologises`).not.toMatch(/\bsorry\b|\boops\b/);
    }
  });

  it("never says 'coming soon'", () => {
    // Every route in the scaffold is empty, so the empty states *are* the product.
    for (const [key, message] of Object.entries(en)) {
      expect(message.toLowerCase(), `${key} is a placeholder`).not.toContain("coming soon");
    }
  });

  it("keeps wgm lowercase and never expands winget", () => {
    for (const [key, message] of Object.entries(en)) {
      expect(message, `${key} capitalises wgm`).not.toMatch(/\bWGM\b|\bWgm\b/);
      expect(message, `${key} expands winget`).not.toContain("Windows Package Manager");
    }
  });

  it("uses the corrected labels rather than the jargon ones", () => {
    // docs/design.md §8: the domain term and the interface term are allowed to differ.
    const values = Object.values(en);

    expect(values).toContain("Export diagnostics");
    expect(values).not.toContain("Export logs");
    expect(values).toContain("Show setup again");
    expect(values).not.toContain("Replay onboarding");
    expect(values).toContain("Check for a new version");
    // "Check for updates" is ambiguous the moment /updates means Package Updates.
    expect(values).not.toContain("Check for updates");
  });
});
