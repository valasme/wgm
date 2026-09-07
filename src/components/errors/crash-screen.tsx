import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { relaunch } from "@tauri-apps/plugin-process";
import { useState } from "react";

import { t } from "@/i18n/t";
import { commands, isAppError } from "@/ipc";
import { collectClientEnvironment } from "@/lib/environment";

/**
 * The crash screen.
 *
 * **Inline styles only, and no imports from the theme layer.** A failure inside the
 * theme or the token layer would otherwise take the crash screen down with it, and
 * there is nothing below this to catch that. Everything here is deliberately plain:
 * one file, no CSS variables, no design system.
 *
 * Three recoveries, in order of how much they cost the user: copy the diagnostics, open
 * the log folder, restart.
 */
export function CrashScreen({ error }: { error: unknown }) {
  const [copied, setCopied] = useState(false);

  const correlationId = isAppError(error) ? error.correlationId : undefined;
  const detail = describe(error);

  const copyDiagnostics = async () => {
    try {
      const summary = await commands.diagnosticsSummary(collectClientEnvironment(), {
        correlationId: correlationId ?? null,
        errorCode: isAppError(error) ? error.code : null,
      });

      await writeText(`${summary}\n\n\`\`\`\n${detail}\n\`\`\`\n`);
      setCopied(true);
    } catch {
      // The clipboard is the last thing left working; if it is not, the log folder
      // button below is still there and the log already has everything.
    }
  };

  return (
    <div style={styles.page}>
      <div style={styles.panel}>
        <h1 style={styles.title}>{t("crash.title")}</h1>
        <p style={styles.body}>{t("crash.body")}</p>

        {correlationId && (
          <code style={styles.correlation}>
            {t("crash.correlationId", { id: correlationId })}
          </code>
        )}

        <pre style={styles.detail}>{detail}</pre>

        <div style={styles.actions}>
          <button type="button" style={styles.primary} onClick={() => void copyDiagnostics()}>
            {copied ? t("crash.copied") : t("crash.copyDiagnostics")}
          </button>
          <button
            type="button"
            style={styles.secondary}
            onClick={() => void commands.openKnownFolder("logs")}
          >
            {t("crash.openLogFolder")}
          </button>
          <button type="button" style={styles.secondary} onClick={() => void relaunch()}>
            {t("crash.restart")}
          </button>
        </div>
      </div>
    </div>
  );
}

function describe(error: unknown): string {
  if (isAppError(error)) {
    return `${error.code} ${JSON.stringify(error.context)}`;
  }
  if (error instanceof Error) {
    return `${error.name}: ${error.message}\n${error.stack ?? ""}`.trim();
  }
  return String(error);
}

// Hardcoded, and hardcoded on purpose: see the component docs. These are the Light
// palette's values, because a crash screen that renders in the wrong scheme is a far
// smaller problem than one that does not render at all.
const styles: Record<string, React.CSSProperties> = {
  page: {
    position: "fixed",
    inset: 0,
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    padding: 32,
    background: "#fcfcfc",
    color: "#1a1a1f",
    fontFamily: "'Segoe UI', system-ui, sans-serif",
    fontSize: 13,
    lineHeight: "20px",
    overflow: "auto",
  },
  panel: {
    display: "flex",
    flexDirection: "column",
    gap: 12,
    maxWidth: 640,
    width: "100%",
  },
  title: { margin: 0, fontSize: 19, lineHeight: "26px", fontWeight: 500 },
  body: { margin: 0, color: "#6b6b76" },
  correlation: {
    fontSize: 11,
    color: "#6b6b76",
    userSelect: "text",
  },
  detail: {
    margin: 0,
    padding: "8px 12px",
    border: "1px solid #e6e6ea",
    borderRadius: 6,
    background: "#f5f5f7",
    fontSize: 11,
    lineHeight: "16px",
    maxHeight: 240,
    overflow: "auto",
    userSelect: "text",
    whiteSpace: "pre-wrap",
    overflowWrap: "anywhere",
  },
  actions: { display: "flex", gap: 8, flexWrap: "wrap" },
  primary: {
    height: 32,
    padding: "0 12px",
    border: "1px solid #2b5fd9",
    borderRadius: 6,
    background: "#2b5fd9",
    color: "#ffffff",
    font: "inherit",
    cursor: "pointer",
  },
  secondary: {
    height: 32,
    padding: "0 12px",
    border: "1px solid #e6e6ea",
    borderRadius: 6,
    background: "#ffffff",
    color: "inherit",
    font: "inherit",
    cursor: "pointer",
  },
};
