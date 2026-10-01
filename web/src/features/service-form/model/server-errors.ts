import type { AddressRow } from "@/entities/service";
import type { FieldError } from "@/shared/api";

export const LISTS_IN_THE_FORM = ["links"];

const PROXY_SETTINGS = /^proxy\.(tls\.)?(email|certificate|key|mode|upstream_verify)$/;

function rowPathOf(field: string, rows: readonly AddressRow[]) {
  const proxied = rows.findIndex((current) => current.proxied);
  if (field === "url") {
    return "rows.0.address";
  }
  if (field.startsWith("environments")) {
    return "rows.0.environments";
  }
  const address = /^addresses\.(.+)$/.exec(field);
  if (address) {
    const index = rows.findIndex((current) => !current.proxied && current.environments.includes(address[1]));
    return `rows.${Math.max(index, 0)}.address`;
  }
  if (proxied >= 0 && field.startsWith("proxy.")) {
    const part = field.replace(/^proxy\./, "").replace(/\[\d+\]$/, "");
    const target = part === "environments" ? "environments" : part === "auth" ? "sign_in" : "address";
    return `rows.${proxied}.${target}`;
  }
  return null;
}

export function formPathOf(field: string, rows: readonly AddressRow[] = []) {
  const settings = PROXY_SETTINGS.exec(field);
  if (settings) {
    return `proxy.${settings[2] === "mode" ? "tls" : settings[2]}`;
  }
  const placed = rowPathOf(field, rows);
  if (placed !== null) {
    return placed;
  }
  const dotted = field.replace(/\[(\d+)\]/g, ".$1");
  const list = dotted.split(".")[0];
  return LISTS_IN_THE_FORM.includes(list) ? dotted : field.replace(/\[\d+\]$/, "");
}

export function byField(errors: FieldError[], fields: string[], rows: readonly AddressRow[] = []) {
  const placed: { path: string; message: string }[] = [];
  const unplaced: FieldError[] = [];
  for (const error of errors) {
    const path = formPathOf(error.field, rows);
    if (fields.includes(path.split(".")[0])) {
      placed.push({ path, message: error.message });
    } else {
      unplaced.push(error);
    }
  }
  return { placed, unplaced };
}
