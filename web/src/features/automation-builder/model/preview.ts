import { quotedCommand } from "@/shared/lib/shell-quote";

import { renderTemplate } from "./placeholders";

export const VARIABLE_PREFIX = "PORTAL_";

export function commandLineOf(script: string, args: string[], samples: Record<string, string>) {
  return quotedCommand(
    script,
    args.map((argument) => renderTemplate(argument, samples)),
  );
}

export function variableOf(field: string) {
  return `${VARIABLE_PREFIX}${field.toUpperCase().replace(/\./g, "_")}`;
}
