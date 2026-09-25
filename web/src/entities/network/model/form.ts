import { z } from "zod";

import type { NetworkSettings } from "./schema";

const IPV4 = /^(25[0-5]|2[0-4]\d|1?\d?\d)(\.(25[0-5]|2[0-4]\d|1?\d?\d)){3}$/;
const IPV6 = /^[0-9a-fA-F:]+$/;

function isAddress(value: string) {
  return IPV4.test(value) || (value.includes(":") && IPV6.test(value));
}

function isNetwork(value: string) {
  const [address, prefix, ...rest] = value.split("/");
  if (rest.length > 0 || !isAddress(address)) {
    return false;
  }
  if (prefix === undefined) {
    return true;
  }
  const bits = Number(prefix);
  return Number.isInteger(bits) && bits >= 0 && bits <= (address.includes(":") ? 128 : 32);
}

function isHttpUrl(value: string) {
  try {
    const url = new URL(value);
    return (url.protocol === "http:" || url.protocol === "https:") && url.hostname !== "";
  } catch {
    return false;
  }
}

export const networkFormSchema = z.object({
  address: z.string().trim().refine(isAddress, "validation.address"),
  port: z.number({ error: "validation.port" }).int("validation.port").min(1, "validation.port").max(65535, "validation.port"),
  public_url: z.string().trim().refine((value) => value === "" || isHttpUrl(value), "validation.publicUrl"),
  trusted_proxies: z.string().refine(
    (value) => proxiesOf(value).every(isNetwork),
    "validation.trustedProxy",
  ),
});

export type NetworkForm = z.infer<typeof networkFormSchema>;

export function proxiesOf(text: string) {
  return text
    .split(/[\n,]/)
    .map((line) => line.trim())
    .filter((line) => line !== "");
}

export function networkFormOf(settings: NetworkSettings): NetworkForm {
  return {
    address: settings.address,
    port: settings.port,
    public_url: settings.public_url ?? "",
    trusted_proxies: settings.trusted_proxies.join("\n"),
  };
}

export function networkRequestOf(form: NetworkForm) {
  return {
    address: form.address.trim(),
    port: form.port,
    public_url: form.public_url.trim() === "" ? null : form.public_url.trim(),
    trusted_proxies: proxiesOf(form.trusted_proxies),
  };
}
