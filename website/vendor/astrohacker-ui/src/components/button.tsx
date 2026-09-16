import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { Slot } from "radix-ui";

import { cn } from "../utils";

/**
 * shadcn Button (new-york), customized for Astrohacker / Tokyo Night.
 * Outline + lg recipes match historical HudCta / brand outline + lg CTAs.
 */
const buttonVariants = cva(
  "inline-flex shrink-0 items-center justify-center gap-2 rounded-md whitespace-nowrap no-underline transition-all outline-none focus-visible:outline-none disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4",
  {
    variants: {
      typography: {
        brand: "font-heading font-bold",
        ui: "font-sans text-sm leading-5 font-semibold tracking-normal normal-case",
      },
      variant: {
        default:
          "bg-primary text-primary-foreground transition-shadow hover:ring-2 hover:ring-accent hover:ring-offset-2 hover:ring-offset-background-highlight focus-visible:ring-2 focus-visible:ring-accent focus-visible:ring-offset-2 focus-visible:ring-offset-background-highlight",
        destructive:
          "bg-destructive text-white hover:bg-destructive/90 focus-visible:ring-2 focus-visible:ring-destructive/40",
        /**
         * Website HUD outline: tinted primary border/fill, not neutral gray.
         * (brand: border-primary/50 bg-primary/10 text-primary hover:border-primary hover:bg-primary/20)
         */
        outline:
          "border border-primary/50 bg-primary/10 text-primary hover:border-primary hover:bg-primary/20",
        secondary:
          "border border-border bg-background-highlight/60 text-foreground hover:border-primary hover:text-primary",
        ghost: "text-foreground hover:bg-accent/10 hover:text-accent",
        link: "bg-transparent font-semibold text-primary underline decoration-primary/40 underline-offset-4 hover:text-accent hover:decoration-accent",
      },
      size: {
        default: "px-6 py-2.5",
        sm: "px-2.5 py-1.5 sm:px-3",
        /**
         * Field-aligned: same outer height as kit `Input` (`h-9`).
         * Use for Input+Button rows (compose, search), not for hero CTAs.
         */
        field: "h-9 px-4",
        /**
         * Hero CTA (website HudCta lg outline): wide tracking + soft cyan glow.
         */
        lg: "px-8 py-3 transition-all duration-300",
        icon: "size-9",
      },
    },
    compoundVariants: [
      {
        typography: "brand",
        size: "default",
        class: "text-sm tracking-[0.15em] uppercase",
      },
      {
        typography: "brand",
        size: "sm",
        class:
          "text-[0.65rem] tracking-[0.12em] uppercase sm:text-xs sm:tracking-[0.14em]",
      },
      {
        typography: "brand",
        size: "field",
        class: "text-sm tracking-[0.12em] uppercase",
      },
      {
        typography: "brand",
        size: "lg",
        class: "text-sm tracking-[0.2em] uppercase",
      },
      {
        variant: "outline",
        size: "lg",
        class: "hover:shadow-[0_0_24px_rgba(125,207,255,0.25)]",
      },
      {
        variant: "link",
        size: "default",
        class: "h-auto p-0 tracking-normal normal-case",
      },
      {
        variant: "link",
        size: "sm",
        class: "h-auto p-0 tracking-normal normal-case",
      },
      {
        variant: "link",
        size: "field",
        class: "h-auto p-0 tracking-normal normal-case",
      },
      {
        variant: "link",
        size: "lg",
        class: "h-auto p-0 text-base tracking-normal normal-case",
      },
    ],
    defaultVariants: {
      typography: "brand",
      variant: "outline",
      size: "default",
    },
  },
);

function Button({
  className,
  variant = "outline",
  size = "default",
  typography = "brand",
  asChild = false,
  ...props
}: React.ComponentProps<"button"> &
  VariantProps<typeof buttonVariants> & {
    asChild?: boolean;
  }): React.JSX.Element {
  const Comp = asChild ? Slot.Root : "button";

  return (
    <Comp
      data-slot="button"
      data-variant={variant}
      data-size={size}
      data-typography={typography}
      className={cn(buttonVariants({ variant, size, typography, className }))}
      {...props}
    />
  );
}

export { Button, buttonVariants };
