import * as SwitchPrimitive from "@radix-ui/react-switch";
import type { ComponentProps } from "react";

import { cn } from "@/lib/cn";

/**
 * A switch.
 *
 * The thumb is a solid shape rather than a tinted one, and the track's on-state uses
 * the Accent — this is one of the three places colour is allowed to mean something.
 * Under `forced-colors` the track falls back to a border, because backgrounds are
 * overridden by the system and borders are re-coloured.
 */
export function Switch({ className, ...props }: ComponentProps<typeof SwitchPrimitive.Root>) {
  return (
    <SwitchPrimitive.Root
      className={cn(
        "peer inline-flex h-5 w-9 shrink-0 items-center rounded-full",
        "border border-border transition-colors duration-100",
        "data-[state=checked]:border-accent data-[state=checked]:bg-accent",
        "data-[state=unchecked]:bg-surface",
        "disabled:cursor-not-allowed disabled:opacity-50",
        "forced-colors:data-[state=checked]:bg-[Highlight]",
        className,
      )}
      {...props}
    >
      <SwitchPrimitive.Thumb
        className={cn(
          "pointer-events-none block size-4 rounded-full",
          "bg-text transition-transform duration-100",
          "data-[state=checked]:translate-x-4 data-[state=checked]:bg-accent-foreground",
          "data-[state=unchecked]:translate-x-0.5",
        )}
      />
    </SwitchPrimitive.Root>
  );
}
