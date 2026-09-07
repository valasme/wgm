import { cn } from "@/lib/cn";

/**
 * The Wordmark — `wgm` set in Instrument Serif Italic.
 *
 * **Not a logo and not a title.** The serif appears in exactly two places in the whole
 * product: here, and the onboarding welcome. Used three times it is a signature; used
 * everywhere it is decoration, and the restraint is the identity.
 *
 * `aria-hidden` because the accessible name of the window is the OS window title, which
 * `useWindowTitle` keeps in step. A screen reader announcing "wgm" on every focus into
 * the Title Bar would be noise.
 */
export function Wordmark({ className }: { className?: string }) {
  return (
    <span
      aria-hidden="true"
      className={cn(
        "select-none font-serif italic leading-none text-text",
        "text-wordmark",
        className,
      )}
    >
      wgm
    </span>
  );
}
