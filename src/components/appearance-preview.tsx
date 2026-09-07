import type { Accent, ColorSchemePreference, Density } from "@/ipc";
import { cn } from "@/lib/cn";
import { resolveColorScheme } from "@/theme/apply-theme";

/**
 * A miniature of the app chrome, drawn in the Appearance being previewed.
 *
 * It carries its own `data-theme`, `data-accent` and `data-density`, so it is painted
 * by the same tokens the real chrome uses rather than a second set of colours nobody
 * has verified (ADR-0002). That is also what makes it show a scheme the app is *not*
 * in — the whole point of a preview.
 *
 * Nothing here is interactive and nothing is announced: it is a picture of the app, and
 * the option's own label is what a screen reader reads. Plain `<div>`s for that reason
 * — reusing the real Sidebar would put a second navigation landmark on the page eight
 * times over.
 */
export function AppearancePreview({
  colorScheme,
  accent,
  density,
  className,
}: {
  colorScheme: ColorSchemePreference;
  accent: Accent;
  density: Density;
  className?: string;
}) {
  return (
    <div
      aria-hidden="true"
      data-theme={resolveColorScheme(colorScheme)}
      data-accent={accent}
      data-density={density}
      className={cn(
        "pointer-events-none flex h-16 w-full flex-col overflow-hidden rounded-sm",
        "border border-border bg-bg",
        className,
      )}
    >
      {/* Title Bar: the toggle square, then the wordmark's ghost. */}
      <div className="flex h-3 shrink-0 items-center gap-1 border-b border-border px-1">
        <div className="size-1.5 rounded-[1px] bg-text-muted/60" />
        <div className="h-1 w-4 rounded-full bg-text-muted/40" />
      </div>

      <div className="flex flex-1">
        {/* Sidebar: one selected row on --surface, the rest muted. Row heights follow
            density, so Compact and Comfortable are visibly different pictures. */}
        <div className="flex w-1/4 flex-col gap-[2px] border-r border-border p-1">
          <div className="h-[calc(var(--row-height)/8)] w-full rounded-[1px] bg-accent" />
          <div className="h-[calc(var(--row-height)/8)] w-full rounded-[1px] bg-text-muted/25" />
          <div className="h-[calc(var(--row-height)/8)] w-full rounded-[1px] bg-text-muted/25" />
        </div>

        {/* Content Area: a heading, two lines, and the Accent on a primary button. */}
        <div className="flex flex-1 flex-col gap-[3px] p-[calc(var(--page-padding)/8)]">
          <div className="h-1 w-2/5 rounded-full bg-text/70" />
          <div className="h-[3px] w-4/5 rounded-full bg-text-muted/40" />
          <div className="h-[3px] w-3/5 rounded-full bg-text-muted/40" />
          <div className="mt-auto h-2 w-6 rounded-[2px] bg-accent" />
        </div>
      </div>
    </div>
  );
}
