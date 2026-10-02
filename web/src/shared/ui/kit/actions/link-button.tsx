import type { VariantProps } from "class-variance-authority";
import Link from "next/link";
import type { ComponentProps } from "react";

import { cn } from "@/shared/lib/cn";

import { buttonVariants } from "./button";

export type LinkButtonProps = ComponentProps<typeof Link> & VariantProps<typeof buttonVariants>;

export function LinkButton({ className, variant = "outline", size = "default", ...props }: LinkButtonProps) {
  return <Link data-slot="link-button" className={cn(buttonVariants({ variant, size }), className)} {...props} />;
}
