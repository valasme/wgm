import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { useEffect, useState } from "react";

import { t } from "@/i18n/t";
import { commands, type ProblemRecord } from "@/ipc";
import { cn } from "@/lib/cn";

import { Button } from "../ui/button";

/**
 * Recent problems.
 *
 * **Errors that are logged but never surfaced are never reported.** The last ten
 * failures, with their timestamps, codes and correlation ids, and a per-row Copy. The
 * audience here is developers: showing them the actual errors raises report quality
 * more than any amount of instructions.
 */
export function RecentProblems() {
  const [problems, setProblems] = useState<ProblemRecord[] | null>(null);

  useEffect(() => {
    void commands.recentProblems().then(setProblems);
  }, []);

  if (problems === null) {
    return null;
  }

  if (problems.length === 0) {
    return (
      <p className="py-2 text-sm text-text-muted">
        {t("settings.advanced.recentProblems.empty")}
      </p>
    );
  }

  return (
    <ul className="flex max-w-[var(--content-max-width)] flex-col divide-y divide-border">
      {problems.map((problem) => (
        <li
          key={`${problem.correlationId}-${problem.timestamp}`}
          className={cn("flex items-center justify-between gap-4 py-2")}
        >
          <div className="flex min-w-0 flex-col gap-0.5">
            <code data-selectable className="truncate text-xs text-text">
              {problem.code}
            </code>
            <span className="text-xs text-text-muted">
              {problem.timestamp} · {problem.correlationId}
            </span>
          </div>

          <Button
            size="small"
            variant="quiet"
            onClick={() => void writeText(formatProblem(problem))}
          >
            {t("settings.advanced.recentProblems.copy")}
          </Button>
        </li>
      ))}
    </ul>
  );
}

/** The same shape as a log line, so a pasted row is greppable against the log file. */
function formatProblem(problem: ProblemRecord): string {
  const context = Object.entries(problem.context)
    .map(([key, value]) => `${key}=${JSON.stringify(value)}`)
    .join(" ");

  return `${problem.timestamp}  ${problem.code}  ${problem.correlationId}  ${context}`.trim();
}
