import { Slot } from "@radix-ui/react-slot";
import { cva, type VariantProps } from "class-variance-authority";
import type { ComponentProps } from "react";

import { cn } from "@/lib/cn";

/**
 * The one button.
 *
 * Colour carries meaning, never mood: only `primary` uses the Accent, and it marks the
 * one action a screen is about. Everything else is neutral, because in an interface
 * this quiet a second coloured button would read as a second primary action.
 */
const button = cva(
  [
    "inline-flex items-center justify-center gap-2 whitespace-nowrap",
    "rounded-sm border border-transparent",
    "text-sm font-medium",
    "transition-colors duration-100",
    // 24px is the accessibility floor and Compact density must not shrink past it.
    "min-h-6",
    "disabled:pointer-events-none disabled:opacity-50",
    "[&_svg]:pointer-events-none [&_svg]:shrink-0",
  ],
  {
    variants: {
      variant: {
        primary: "bg-accent text-accent-foreground hover:opacity-90",
        secondary: "border-border bg-surface text-text hover:border-text-muted",
        ghost: "text-text hover:bg-surface",
        // Not a red button: nothing in the palette is red, and a destructive action is
        // gated by a confirmation rather than by looking dangerous.
        quiet: "text-text-muted hover:text-text hover:bg-surface",
      },
      size: {
        default: "h-8 px-3",
        small: "h-7 px-2 text-xs",
        icon: "size-8",
        iconSmall: "size-6",
      },
    },
    defaultVariants: {
      variant: "secondary",
      size: "default",
    },
  },
);

export type ButtonProps = ComponentProps<"button"> &
  VariantProps<typeof button> & {
    /** Render as the child element — a router `<Link>`, typically. */
    asChild?: boolean;
  };

export function Button({ className, variant, size, asChild = false, ...props }: ButtonProps) {
  const Component = asChild ? Slot : "button";

  return (
    <Component
      className={cn(button({ variant, size }), className)}
      // A `<button>` inside a form defaults to submitting it, which is never what a
      // button in this app means.
      type={asChild ? undefined : (props.type ?? "button")}
      {...props}
    />
  );
}

export { button as buttonVariants };
