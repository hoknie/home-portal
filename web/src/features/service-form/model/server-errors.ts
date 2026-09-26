import type { FieldError } from "@/shared/api";

export const LISTS_IN_THE_FORM = ["links"];

export const PROXY_PREFIX = /^proxy\.(tls\.)?/;

export function formPathOf(field: string) {
  if (PROXY_PREFIX.test(field)) {
    return field.replace(PROXY_PREFIX, "publication.").replace(/\[\d+\]$/, "");
  }
  const dotted = field.replace(/\[(\d+)\]/g, ".$1");
  const list = dotted.split(".")[0];
  return LISTS_IN_THE_FORM.includes(list) ? dotted : field.replace(/\[\d+\]$/, "");
}

export function byField(errors: FieldError[], fields: string[]) {
  const placed: { path: string; message: string }[] = [];
  const unplaced: FieldError[] = [];
  for (const error of errors) {
    const path = formPathOf(error.field);
    if (fields.includes(path.split(".")[0])) {
      placed.push({ path, message: error.message });
    } else {
      unplaced.push(error);
    }
  }
  return { placed, unplaced };
}
