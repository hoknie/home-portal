import { z } from "zod";

import type { Service } from "./schema";

export const INTERNET = "internet";

export const addressRowSchema = z.object({
  environments: z.array(z.string()),
  sign_in: z.boolean(),
  address: z.string().trim(),
  proxied: z.boolean(),
  locked: z.boolean(),
  auth: z.array(z.string()).nullable(),
});

export type AddressRow = z.infer<typeof addressRowSchema>;

export type AddressFields = {
  url: string;
  addresses: Record<string, string>;
  environments: string[] | null;
  proxy: { host: string; environments: string[]; auth: string[] } | null;
};

const HOST_LABEL = /^[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?$/;

export function choicesOf(configured: readonly string[]) {
  return configured.includes(INTERNET) ? [...configured] : [...configured, INTERNET];
}

export function row(fields: Partial<AddressRow> = {}): AddressRow {
  return { environments: [], sign_in: false, address: "", proxied: false, locked: false, auth: null, ...fields };
}

function sameSet(left: readonly string[], right: readonly string[]) {
  return left.length === right.length && left.every((name) => right.includes(name));
}

function inOrder(names: Iterable<string>, all: readonly string[]) {
  const wanted = new Set(names);
  return [...all.filter((name) => wanted.has(name)), ...[...wanted].filter((name) => !all.includes(name))];
}

export function rowsOf(service: Pick<Service, "url" | "addresses" | "environments" | "proxy">, all: readonly string[]): AddressRow[] {
  const visible = service.environments ?? [...all];
  const published = service.proxy?.environments ?? [];
  const direct = new Map<string, string[]>();
  const hidden: AddressRow[] = [];
  for (const [environment, address] of Object.entries(service.addresses)) {
    if (!visible.includes(environment)) {
      hidden.push(row({ environments: [environment], address, locked: true }));
      continue;
    }
    direct.set(address, [...(direct.get(address) ?? []), environment]);
  }
  const taken = new Set([...Object.keys(service.addresses), ...published]);
  const rows = [row({ address: service.url, environments: inOrder(visible.filter((name) => !taken.has(name)), all) })];
  for (const [address, environments] of direct) {
    rows.push(row({ address, environments: inOrder(environments, all) }));
  }
  if (service.proxy) {
    const auth = service.proxy.auth;
    const plain = auth.length === 0 || sameSet(auth, published);
    rows.push(
      row({
        address: `https://${service.proxy.host}`,
        environments: inOrder(published, all),
        sign_in: auth.length > 0,
        proxied: true,
        locked: !plain,
        auth: plain ? null : [...auth],
      }),
    );
  }
  return [...rows, ...hidden];
}

export function hostOf(address: string) {
  try {
    return new URL(address.trim()).hostname;
  } catch {
    return "";
  }
}

export function fieldsOf(rows: readonly AddressRow[], all: readonly string[]): AddressFields {
  const [main, ...others] = rows;
  const addresses: Record<string, string> = {};
  let proxy: AddressFields["proxy"] = null;
  const shown = new Set(main?.environments ?? []);
  for (const current of others) {
    if (current.proxied) {
      proxy = {
        host: hostOf(current.address),
        environments: [...current.environments],
        auth: current.auth ?? (current.sign_in ? [...current.environments] : []),
      };
      current.environments.forEach((name) => shown.add(name));
      continue;
    }
    for (const name of current.environments) {
      addresses[name] = current.address.trim();
      if (!current.locked) {
        shown.add(name);
      }
    }
  }
  const everywhere = all.every((name) => shown.has(name));
  return { url: (main?.address ?? "").trim(), addresses, environments: everywhere ? null : inOrder(shown, all), proxy };
}

export function publishedAddressAccepted(address: string) {
  try {
    const url = new URL(address.trim());
    return (
      url.protocol === "https:" &&
      url.port === "" &&
      (url.pathname === "/" || url.pathname === "") &&
      url.search === "" &&
      url.hash === "" &&
      url.hostname.length <= 253 &&
      url.hostname.split(".").every((label) => HOST_LABEL.test(label))
    );
  } catch {
    return false;
  }
}

export type RowProblem = { index: number; field: keyof AddressRow; message: string };

export function rowProblems(rows: readonly AddressRow[], all: readonly string[], accepted: (address: string, main: boolean) => boolean): RowProblem[] {
  const problems: RowProblem[] = [];
  const holder = new Map<string, number>();
  let proxied = rows.findIndex((current) => current.locked && current.proxied);
  rows.forEach((current, index) => {
    if (current.locked) {
      return;
    }
    if (index === 0 && current.proxied) {
      problems.push({ index, field: "proxied", message: "validation.mainNotProxied" });
    }
    if (!current.proxied && !accepted(current.address, index === 0)) {
      problems.push({ index, field: "address", message: index === 0 ? "validation.url" : "validation.address" });
    }
    if (index > 0 && current.environments.length === 0) {
      problems.push({ index, field: "environments", message: "validation.rowEnvironments" });
    }
    for (const name of current.environments) {
      if (holder.has(name)) {
        problems.push({ index, field: "environments", message: "validation.rowEnvironmentTaken" });
      }
      holder.set(name, index);
    }
    if (current.sign_in && !current.proxied) {
      problems.push({ index, field: "sign_in", message: "validation.signInNeedsProxy" });
    }
    if (current.proxied && index > 0) {
      if (proxied >= 0) {
        problems.push({ index, field: "proxied", message: "validation.oneProxiedRow" });
      }
      proxied = index;
      if (!publishedAddressAccepted(current.address)) {
        problems.push({ index, field: "address", message: "validation.proxyHost" });
      }
    }
  });
  if (rows.filter((current) => !current.locked).length > all.length) {
    problems.push({ index: rows.length - 1, field: "environments", message: "validation.tooManyRows" });
  }
  return problems;
}
