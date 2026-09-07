import { t } from "@/i18n/t";

/**
 * The loading state.
 *
 * A live region rather than a spinner: there is nothing to look at yet, and a screen
 * reader needs to hear that something is happening. Motion is one short pulse and is
 * suppressed entirely under reduced motion, along with everything else.
 */
export function LoadingState({ label = t("state.loading") }: { label?: string }) {
  return (
    <div
      role="status"
      aria-live="polite"
      className="flex flex-1 items-center justify-center p-12 text-sm text-text-muted"
    >
      {label}
    </div>
  );
}
