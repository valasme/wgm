import type { ReactNode, Ref } from "react";

import { cn } from "@/lib/cn";

/**
 * The empty state.
 *
 * **An empty screen is an invitation to act, not a status report.** Every route in the
 * scaffold is empty, so the empty states *are* the product right now — and they cost
 * the same to write well. There is no "coming soon" copy anywhere in wgm.
 *
 * The heading takes the ref, because focus moves to the heading that changed on
 * navigation. No serif here: the wordmark and the onboarding welcome are its only two
 * placements.
 */
export function EmptyState({
  heading,
  body,
  action,
  headingRef,
  headingLevel = 1,
  icon,
}: {
  heading: string;
  body?: string;
  action?: ReactNode;
  headingRef?: Ref<HTMLHeadingElement>;
  headingLevel?: 1 | 2;
  icon?: ReactNode;
}) {
  const Heading = headingLevel === 1 ? "h1" : "h2";

  return (
    <div
      className={cn(
        "flex flex-1 flex-col items-center justify-center gap-3 text-center",
        "p-[calc(var(--page-padding)*1.5)]",
      )}
    >
      {icon && <div className="text-text-muted">{icon}</div>}

      <Heading ref={headingRef} className="text-lg font-medium text-text outline-none">
        {heading}
      </Heading>

      {body && <p className="max-w-[52ch] text-sm text-text-muted">{body}</p>}

      {action && <div className="mt-2">{action}</div>}
    </div>
  );
}
