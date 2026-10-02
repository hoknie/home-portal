import { createElement } from "react";

import { BLOCK_ICONS } from "../../../model/block-icons";

export function BlockIcon({ name, className }: { name: string | null; className?: string }) {
  const icon = name === null ? undefined : BLOCK_ICONS[name];
  return icon ? createElement(icon, { className, "aria-hidden": true }) : null;
}
