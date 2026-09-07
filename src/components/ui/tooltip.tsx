import * as TooltipPrimitive from "@radix-ui/react-tooltip";
import type { ComponentProps, ReactNode } from "react";

import { cn } from "@/lib/cn";

/**
 * A tooltip.
 *
 * **Never the accessible name.** Tooltips are not reliably announced, so every element
 * that carries one — a Rail item especially — also carries an `aria-label`. This is a
 * sighted-pointer affordance and nothing more.
 */
export const TooltipProvider = TooltipPrimitive.Provider;

export function Tooltip({
  children,
  content,
  side = "right",
  ...props
}: ComponentProps<typeof TooltipPrimitive.Root> & {
  children: ReactNode;
  content: ReactNode;
  side?: "top" | "right" | "bottom" | "left";
}) {
  return (
    <TooltipPrimitive.Root {...props}>
      <TooltipPrimitive.Trigger asChild>{children}</TooltipPrimitive.Trigger>
      <TooltipPrimitive.Portal>
        <TooltipPrimitive.Content
          side={side}
          sideOffset={8}
          className={cn(
            "z-50 rounded-sm border border-border bg-bg px-2 py-1",
            "text-xs text-text shadow-md",
          )}
        >
          {content}
        </TooltipPrimitive.Content>
      </TooltipPrimitive.Portal>
    </TooltipPrimitive.Root>
  );
}
