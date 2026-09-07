import type { Ref } from "react";

import { t } from "@/i18n/t";

/**
 * The welcome step.
 *
 * **The second and last placement of the serif.** The Title Bar wordmark is the other
 * one. Used three times it is a signature; used everywhere it is decoration, and the
 * restraint is the whole identity.
 */
export function StepWelcome({ headingRef }: { headingRef: Ref<HTMLHeadingElement> }) {
  return (
    <div className="flex max-w-[52ch] flex-col items-center gap-4 text-center">
      <h1
        ref={headingRef}
        className="font-serif text-[64px] italic leading-none text-text outline-none"
      >
        {t("onboarding.welcome.title")}
      </h1>

      <p className="text-sm text-text-muted">{t("onboarding.welcome.body")}</p>
    </div>
  );
}
