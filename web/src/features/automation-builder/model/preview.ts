import { renderTemplate } from "./placeholders";

export const VARIABLE_PREFIX = "PORTAL_";

export function quoted(argument: string) {
  return `'${argument.replace(/'/g, `'\\''`)}'`;
}

export function commandLineOf(script: string, args: string[], samples: Record<string, string>) {
  return [script, ...args.map((argument) => quoted(renderTemplate(argument, samples)))].join(" ");
}

export function variableOf(field: string) {
  return `${VARIABLE_PREFIX}${field.toUpperCase().replace(/\./g, "_")}`;
}
