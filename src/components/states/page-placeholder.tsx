import type { ReactNode } from "react";
import type { MessageKey } from "@/i18n/t";
import { t } from "@/i18n/t";
import { useHeadingFocus, useWindowTitle } from "@/lib/focus";

import { EmptyState } from "./empty-state";

/**
 * A route that has no data behind it yet.
 *
 * This is not a "coming soon" screen — it renders the route's **real** empty state,
 * with the copy fixed in docs/design.md §8. The scaffold is almost entirely empty
 * states, so they are the product today.
 *
 * It also does the two things every route owes: set the OS window title, and take focus
 * on the heading that changed.
 */
export function PagePlaceholder({
  title,
  heading,
  body,
  action,
  icon,
}: {
  /** The window title fragment: `Search · wgm`. */
  title: MessageKey;
  heading: MessageKey;
  body?: MessageKey;
  action?: ReactNode;
  icon?: ReactNode;
}) {
  useWindowTitle(title);
  const headingRef = useHeadingFocus<HTMLHeadingElement>();

  return (
    <EmptyState
      headingRef={headingRef}
      heading={t(heading)}
      body={body ? t(body) : undefined}
      action={action}
      icon={icon}
    />
  );
}
