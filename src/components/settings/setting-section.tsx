import type { ReactNode, Ref } from "react";

import { cn } from "@/lib/cn";

/**
 * A settings page.
 *
 * Settings is the one place with two heading levels: the layout owns `<h1>Settings`
 * and each page owns an `<h2>`. Focus on navigation moves to *the heading that
 * changed* — sending it to the `<h1>` would announce "Settings" four times in a row
 * and never name the page the user actually opened.
 */
export function SettingSection({
  heading,
  headingRef,
  children,
  className,
}: {
  heading: string;
  headingRef?: Ref<HTMLHeadingElement>;
  children: ReactNode;
  className?: string;
}) {
  return (
    <section className={cn("flex flex-col", className)}>
      <h2 ref={headingRef} className="mb-2 text-md font-medium text-text outline-none">
        {heading}
      </h2>

      <div className="flex flex-col divide-y divide-border">{children}</div>
    </section>
  );
}
