import { describe, expect, it } from "vitest";

import { apiSamples } from "@/shared/api";

import { dnsFormOf, dnsRequestOf } from "./form";
import { answerIn, dnsSchema } from "./schema";

const dns = dnsSchema.parse(apiSamples.dns);

describe("dns", () => {
  it("the sample parses with its transports and answers per environment", () => {
    expect(dns.plain.listening).toBe(false);
    expect(dns.tls.listening).toBe(true);
    expect(answerIn(dns, "jellyfin.home", "vpn")).toEqual([{ type: "A", value: "10.8.0.1" }]);
    expect(answerIn(dns, "nas.home", "vpn")).toEqual([]);
    expect(dns.unaddressed).toEqual(["office"]);
  });

  it("the form has one address row per environment and writes only the filled ones", () => {
    const form = dnsFormOf(dns);
    expect(Object.keys(form.addresses)).toEqual(["local", "office", "vpn"]);
    expect(form.addresses.vpn).toBe("10.8.0.1");
    const request = dnsRequestOf({ ...form, addresses: { ...form.addresses, local: "192.168.1.60, fd00::60" } }, dns);
    expect(request.addresses).toEqual({ local: ["192.168.1.60", "fd00::60"], vpn: ["10.8.0.1"] });
    expect(request.https.host).toBeNull();
  });
});
