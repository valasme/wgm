import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { useState } from "react";
import { TitleBar } from "@/components/chrome/title-bar";
import { StepAppearance } from "@/components/onboarding/step-appearance";
import { StepPrivacy } from "@/components/onboarding/step-privacy";
import { StepReady } from "@/components/onboarding/step-ready";
import { StepWelcome } from "@/components/onboarding/step-welcome";
import { Button } from "@/components/ui/button";
import { t } from "@/i18n/t";
import { cn } from "@/lib/cn";
import { useHeadingFocus, useWindowTitle } from "@/lib/focus";
import { useWorkspaceStore } from "@/stores/workspace-store";

export const Route = createFileRoute("/onboarding")({
  component: Onboarding,
});

const STEPS = [StepWelcome, StepAppearance, StepPrivacy, StepReady];

/**
 * First-run setup.
 *
 * **It does not trap focus.** It is a route with nothing behind it, so a trap buys
 * nothing — and it would lock the user away from the Window Controls, leaving them
 * unable to minimise or close the app during setup. The Title Bar is rendered here for
 * the same reason.
 *
 * Skippable, and Defaults apply if skipped. Progress is recorded as a version string
 * rather than a boolean, so a future major release can replay a single "what's new"
 * step; and it lives in Workspace State, so importing someone else's settings never
 * skips your setup.
 */
function Onboarding() {
  useWindowTitle("app.name");

  const navigate = useNavigate();
  const completeOnboarding = useWorkspaceStore((state) => state.completeOnboarding);
  const [step, setStep] = useState(0);

  // Focus starts on the step heading. New ref per step, so each step announces itself.
  const headingRef = useHeadingFocus<HTMLHeadingElement>();

  const finish = async () => {
    // If Settings cannot be written, onboarding still finishes — the store swallows the
    // failure and the Ephemeral Mode banner explains it. Replaying setup on every
    // launch is a maddening symptom for a small failure.
    await completeOnboarding();
    await navigate({ to: "/search" });
  };

  const Step = STEPS[step] ?? StepWelcome;
  const last = step === STEPS.length - 1;

  return (
    <div className="flex h-full flex-col overflow-hidden">
      <TitleBar />

      <main
        id="main"
        tabIndex={-1}
        className="flex flex-1 flex-col items-center justify-center gap-8 overflow-y-auto p-12 outline-none"
      >
        <Step key={step} headingRef={headingRef} />

        <div className="flex flex-col items-center gap-6">
          <ol
            className="flex items-center gap-2"
            aria-label={t("onboarding.progress", { step: step + 1, total: STEPS.length })}
          >
            {STEPS.map((_, index) => (
              <li
                // biome-ignore lint/suspicious/noArrayIndexKey: a fixed, ordered list whose members have no identity beyond their position
                key={index}
                aria-current={index === step ? "step" : undefined}
                className={cn(
                  "size-1.5 rounded-full",
                  index === step ? "bg-accent" : "bg-border",
                )}
              />
            ))}
          </ol>

          <div className="flex items-center gap-2">
            {step > 0 && (
              <Button variant="ghost" onClick={() => setStep((current) => current - 1)}>
                {t("onboarding.back")}
              </Button>
            )}

            <Button
              variant="primary"
              onClick={() => (last ? void finish() : setStep((current) => current + 1))}
            >
              {last ? t("onboarding.finish") : t("onboarding.next")}
            </Button>
          </div>

          {/* A real focusable control, not a corner affordance. */}
          {!last && (
            <Button variant="quiet" size="small" onClick={() => void finish()}>
              {t("onboarding.skip")}
            </Button>
          )}
        </div>
      </main>
    </div>
  );
}
