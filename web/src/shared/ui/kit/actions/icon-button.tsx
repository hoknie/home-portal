import type { ComponentProps } from "react";

import { Button } from "./button";

export type IconButtonProps = Omit<ComponentProps<typeof Button>, "size" | "aria-label"> & {
  label: string;
  size?: "icon" | "icon-xs" | "icon-sm" | "icon-lg";
};

export function IconButton({ label, size = "icon", variant = "ghost", ...props }: IconButtonProps) {
  return <Button aria-label={label} title={label} size={size} variant={variant} {...props} />;
}
